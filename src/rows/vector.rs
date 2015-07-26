use alloc::heap;
use std::marker;
use std::mem;
use std::slice;
use std::ptr;

use common::constant::VECTOR_SIZE;
use common::StringSlice;
use expr::Datum;
use intrinsics::sse;
use types::*;

pub trait Vector : HasTy {
  fn size(&self) -> usize;
  fn as_ptr(&self) -> *const u8;
  fn as_mut_ptr(&mut self) ->*mut u8;
  fn is_const(&self) -> bool;
}

pub struct ArrayVector<'a> {
  ptr: *mut u8,
  data_ty: Ty,
  _marker: marker::PhantomData<&'a ()>
}

impl<'a> ArrayVector<'a> {
  pub fn new(data_ty: Ty) -> ArrayVector<'a> {
    let alloc_size = sse::compute_aligned_size(
      data_ty.bytes_len() as usize * VECTOR_SIZE);

    let ptr = unsafe {
      heap::allocate(alloc_size, sse::ALIGNED_SIZE)
    };

    ArrayVector {
      ptr: ptr,
      data_ty: data_ty,
      _marker: marker::PhantomData
    }
  }  
}

impl<'a> Drop for ArrayVector<'a> {
  fn drop(&mut self) {
    unsafe {
      let alloc_size = sse::compute_aligned_size(
        self.data_ty.bytes_len() as usize * VECTOR_SIZE);

      heap::deallocate(self.ptr as *mut u8, alloc_size, sse::ALIGNED_SIZE);
    }
  }
}

impl<'a> Vector for ArrayVector<'a> {
  #[inline]
  fn size(&self) -> usize { VECTOR_SIZE }

  #[inline]
  fn as_ptr(&self) -> *const u8 {
    self.ptr
  }

  #[inline]
  fn as_mut_ptr(&mut self) -> *mut u8 {
    self.ptr
  }

  #[inline]
  fn is_const(&self) -> bool { false }
}

impl<'a> HasTy for ArrayVector<'a> {
  fn data_ty(&self) -> &Ty {
    &self.data_ty
  }
}

pub struct ConstVector {
  value: [u8; 16],
  data_ty: Ty,
  datum: Datum
}

impl ConstVector {
  pub fn new(datum: Datum) -> ConstVector {
    let value: [u8; 16] = unsafe { mem::zeroed() };
    unsafe {
      match datum {
        Datum::Bool(v) => ptr::write(value.as_ptr() as *mut BOOL, v),
        Datum::Int1(v) => ptr::write(value.as_ptr() as *mut INT1, v),
        Datum::Int2(v) => ptr::write(value.as_ptr() as *mut INT2, v),
        Datum::Int4(v) => ptr::write(value.as_ptr() as *mut INT4, v),
        Datum::Int8(v) => ptr::write(value.as_ptr() as *mut INT8, v),
        Datum::Float4(v) => ptr::write(value.as_ptr() as *mut FLOAT4, v),
        Datum::Float8(v) => ptr::write(value.as_ptr() as *mut FLOAT8, v),
        Datum::Time(v) => ptr::write(value.as_ptr() as *mut TIME, v),        
        Datum::Date(v) => ptr::write(value.as_ptr() as *mut DATE, v),  
        Datum::Timestamp(v) => ptr::write(value.as_ptr() as *mut TIMESTAMP, v),  
        // Datum::Interval(v) => ptr::write(value.as_ptr() as *mut INTERVAL_T, v),  
        // Datum::Char(v) => ptr::write(value.as_ptr() , v),  
        Datum::Text(ref v) => {
          let text: TEXT = StringSlice::new_from_str(v.as_str());
          ptr::write(value.as_ptr() as *mut TEXT, text);
        }
        // Datum::Varchar(v) => ptr::write(value.as_ptr(), v),  
        // Datum::Blob(v) => ptr::write(value.as_ptr(), v),  
        _ => panic!("not support type")
      }
    }
    
    ConstVector {
      value: value,      
      data_ty: datum.data_ty().clone(),
      datum: datum
    }
  }
}

impl HasTy for ConstVector {
  fn data_ty(&self) -> &Ty {
    &self.data_ty
  }
}

impl Vector for ConstVector {
  #[inline]
  fn size(&self) -> usize { 1 }

  #[inline]
  fn as_ptr(&self) -> *const u8 {
    self.value.as_ptr() as *const u8
  }

  #[inline]
  fn as_mut_ptr(&mut self) -> *mut u8 {
    self.value.as_ptr() as *mut u8
  }

  #[inline]
  fn is_const(&self) -> bool { false }
}

#[inline]
pub fn first_value<T>(v: &Vector) -> &T {
  let array = unsafe {
    slice::from_raw_parts(v.as_ptr() as *const T, 1) as &[T]
  };

  &array[0]
}

#[inline]
pub fn as_array<T>(v: &Vector) -> &[T] {
  unsafe {
    slice::from_raw_parts(v.as_ptr() as *const T, VECTOR_SIZE)
  }    
}

#[inline]
pub fn as_mut_array<T>(v: &mut Vector) -> &mut [T] {
  unsafe {
    slice::from_raw_parts_mut(v.as_mut_ptr() as *mut T, VECTOR_SIZE)
  }    
}

/// Return a filled array vector from a list of values
pub fn from_vec<'a, T>(data_ty: &Ty, values: &Vec<T>) -> ArrayVector<'a>
  where T: Copy {

  let mut vec = ArrayVector::new(data_ty.clone());
  {
    let mut array: &mut [T] = as_mut_array(&mut vec);
    for x in 0..values.len() {
      array[x] = values[x];
    }
  }

  vec
}

#[test]
fn test_const_vector() {
  let bool_vec: &Vector = &ConstVector::new(Datum::Bool(true));
  assert_eq!(BOOL_TY, bool_vec.data_ty());
  assert_eq!(1, bool_vec.size());
  assert_eq!(true, *first_value(bool_vec));

  let int1_vec: &Vector = &ConstVector::new(Datum::Int1(7));
  assert_eq!(INT1_TY, int1_vec.data_ty());
  assert_eq!(1, int1_vec.size());
  assert_eq!(7, *first_value(int1_vec));

  let int2_vec: &Vector = &ConstVector::new(Datum::Int2(17));
  assert_eq!(INT2_TY, int2_vec.data_ty());
  assert_eq!(1, int2_vec.size());
  assert_eq!(17, *first_value(int2_vec));

  let int4_vec: &Vector = &ConstVector::new(Datum::Int4(178910));
  assert_eq!(INT4_TY, int4_vec.data_ty());
  assert_eq!(1, int4_vec.size());
  assert_eq!(178910, *first_value(int4_vec));

  let int8_vec: &Vector = &ConstVector::new(Datum::Int8(981627341));
  assert_eq!(INT8_TY, int8_vec.data_ty());
  assert_eq!(1, int8_vec.size());
  assert_eq!(981627341, *first_value(int8_vec));

  let float4_vec: &Vector = &ConstVector::new(Datum::Float4(3.14f32));
  assert_eq!(FLOAT4_TY, float4_vec.data_ty());
  assert_eq!(1, float4_vec.size());
  assert_eq!(3.14f32, *first_value(float4_vec));

  let float8_vec: &Vector = &ConstVector::new(Datum::Float8(87123.1452f64));
  assert_eq!(FLOAT8_TY, float8_vec.data_ty());
  assert_eq!(1, float8_vec.size());
  assert_eq!(87123.1452f64, *first_value(float8_vec));

  let text_vec: &Vector = &ConstVector::new(Datum::Text("hyunsik".to_string()));
  assert_eq!(TEXT_TY, text_vec.data_ty());
  assert_eq!(1, text_vec.size());
  let expected = StringSlice::new_from_str("hyunsik");
  assert_eq!(expected, *first_value(text_vec));
}