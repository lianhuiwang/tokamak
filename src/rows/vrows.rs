use bytesize::ByteSize;
use types::*;
use common::constant::VECTOR_SIZE;
use intrinsics::sse;
use memutil::Arena;
use schema::Schema;
use rows::vector::{Vector, as_array, as_mut_array};
use rows::{AsRowBlock, RowBlock, RowBlockWriter};

use alloc::heap;
use std::marker;
use std::slice;


pub struct BorrowedVRowBlock<'a> {
  schema: Schema,
  vectors: Vec<&'a Vector>,
  selected: Vec<bool>
}

impl<'a> BorrowedVRowBlock<'a> {
  pub fn new(schema: &Schema) -> BorrowedVRowBlock<'a> {
    BorrowedVRowBlock {
      schema: schema.clone(), 
      vectors: Vec::new(), 
      selected: Vec::new()
    }
  }

  #[inline]
  fn set_vector(&mut self, vec: &'a Vector) {
    self.vectors.push(vec);
  }
}

impl<'a> AsRowBlock for BorrowedVRowBlock<'a> {
  fn as_reader(&self) -> &RowBlock {
    self
  }
}

impl<'a> RowBlock for BorrowedVRowBlock<'a> {
  #[inline]
  fn schema(&self) -> &Schema {
    &self.schema
  }

  #[inline]
  fn column_num(&self) -> usize {
    self.schema.size()
  }

  #[inline]
  fn vector(&self, col_id: usize) -> &Vector {
    self.vectors[col_id]
  }

  fn selected(&self) -> &Vec<bool> {
    &self.selected
  }

  fn selected_mut(&mut self) -> &mut Vec<bool> {
    &mut self.selected
  }

  #[inline]
  fn get_int1(&self, row_id: usize, col_id: usize ) -> INT1 {      
    let v : &[INT1] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }  

  #[inline]
  fn get_int2(&self, row_id: usize, col_id: usize ) -> INT2 {      
    let v : &[INT2] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_int4(&self, row_id: usize, col_id: usize ) -> INT4 {      
    let v : &[INT4] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_int8(&self, row_id: usize, col_id: usize ) -> INT8 {      
    let v : &[INT8] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_float4(&self, row_id: usize, col_id: usize ) -> FLOAT4 {      
    let v : &[FLOAT4] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_float8(&self, row_id: usize, col_id: usize ) -> FLOAT8 {      
    let v : &[FLOAT8] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_date(&self, row_id: usize, col_id: usize ) -> DATE {      
    let v : &[DATE] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_time(&self, row_id: usize, col_id: usize ) -> TIME {      
    let v : &[TIME] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_timestamp(&self, row_id: usize, col_id: usize ) -> TIMESTAMP {      
    let v : &[TIMESTAMP] = as_array(self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  fn get_text(&self, row_id: usize, col_id: usize) -> &TEXT {
    let v : &[TEXT] = as_array(self.vectors[col_id]);
    unsafe {
      v.get_unchecked(row_id)
    }
  }
}



/// Borrowed vector
pub struct PtrVector<'a> {
  ptr: *mut u8,
  size: usize,
  data_type: DataTy,
  _marker: marker::PhantomData<&'a ()>
}

impl<'a> PtrVector<'a> {
  pub fn new(ptr: *mut u8, size: usize, data_type: DataTy) -> PtrVector<'a> {
    PtrVector {
      ptr: ptr, 
      size: size,
      data_type: data_type, 
      _marker: marker::PhantomData
    }
  }  
}

impl<'a> Vector for PtrVector<'a> {
  #[inline]
  fn size(&self) -> usize {self.size}

  #[inline]
  fn as_ptr(&self) -> *const u8 {
    self.ptr
  }

  #[inline]
  fn as_mut_ptr(&mut self) -> *mut u8 {
    self.ptr
  }

  fn is_const(&self) -> bool { false }
}

impl<'a> HasDataTy for PtrVector<'a> {
  fn data_ty(&self) -> &DataTy {
    &self.data_type
  }
}

pub struct HeapVRowBlock<'a> {
  schema: Schema,  
  type_lengths: Vec<u32>,
  ptr: *mut u8,
  vectors: Vec<PtrVector<'a>>,
  selected: Vec<bool>,
  arena: Arena<'a>
}

impl<'a> HeapVRowBlock<'a> {

  pub fn new(schema: &Schema) -> HeapVRowBlock<'a> {

    let mut fixed_area_size: usize = 0;    
    let mut type_lengths: Vec<u32> = Vec::new();

    for c in schema.columns() {
      let bytes_len = c.data_ty.bytes_len();      
      type_lengths.push(bytes_len);

      fixed_area_size += 
        sse::compute_aligned_size(bytes_len as usize * VECTOR_SIZE);
    }

    let fixed_area_ptr = unsafe {
      heap::allocate(fixed_area_size, sse::ALIGNED_SIZE)
    };

    
    let mut vectors: Vec<PtrVector> = Vec::with_capacity(schema.size());
    let mut last_ptr = fixed_area_ptr as usize;

    for x in 0..schema.size() {      
      vectors.push(
        PtrVector::new(last_ptr as *mut u8, VECTOR_SIZE, schema.column(x).data_ty));

      let vector_size = 
        sse::compute_aligned_size(schema.column(x).data_ty.bytes_len() as usize * VECTOR_SIZE);
      last_ptr = last_ptr + vector_size;
    }

    HeapVRowBlock {
      schema: schema.clone(), 
      type_lengths: type_lengths, 
      ptr: fixed_area_ptr, 
      vectors: vectors,
      selected: Vec::new(),
      arena: Arena::new(ByteSize::kb(4).as_usize())
    }
  }  
}

impl<'a> AsRowBlock for HeapVRowBlock<'a> {
  fn as_reader(&self) -> &RowBlock {
    self
  }
}



impl<'a> RowBlockWriter for HeapVRowBlock<'a> {
  #[inline]
  fn put_int1(&mut self, row_id: usize, col_id: usize, value: INT1) {      
    let v : &mut [INT1] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

  #[inline]
  fn put_int2(&mut self, row_id: usize, col_id: usize, value: INT2) {      
    let v : &mut [INT2] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    #[inline]
  fn put_int4(&mut self, row_id: usize, col_id: usize, value: INT4) {      
    let v : &mut [INT4] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

  #[inline]
  fn put_int8(&mut self, row_id: usize, col_id: usize, value: INT8) {      
    let v : &mut [INT8] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    #[inline]
  fn put_float4(&mut self, row_id: usize, col_id: usize, value: FLOAT4) {      
    let v : &mut [FLOAT4] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    #[inline]
  fn put_float8(&mut self, row_id: usize, col_id: usize, value: FLOAT8) {      
    let v : &mut [FLOAT8] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    #[inline]
  fn put_date(&mut self, row_id: usize, col_id: usize, value: DATE) {      
    let v : &mut [DATE] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    #[inline]
  fn put_time(&mut self, row_id: usize, col_id: usize, value: TIME) {      
    let v : &mut [TIME] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }


  #[inline]
  fn put_timestamp(&mut self, row_id: usize, col_id: usize, value: TIMESTAMP) {      
    let v : &mut [TIMESTAMP] = as_mut_array(&mut self.vectors[col_id]);
    unsafe{
      (*v.get_unchecked_mut(row_id)) = value;        
    }
  }

    fn put_text(&mut self, row_id: usize, col_id: usize, value: &str) {
    let v : &mut [TEXT] = as_mut_array(&mut self.vectors[col_id]);

    let str_ptr = self.arena.alloc_str(value);

    v[row_id].set_ptr(str_ptr);
    v[row_id].set_len(value.len() as i32);
  }
}

impl<'a> RowBlock for HeapVRowBlock<'a> {
  fn schema(&self) -> &Schema {
    &self.schema
  }

  fn column_num(&self) -> usize {
    self.schema.size()
  }

  fn vector(&self, col_id: usize) -> &Vector {
    &self.vectors[col_id]
  }

  fn selected(&self) -> &Vec<bool> {
    &self.selected
  }

  fn selected_mut(&mut self) -> &mut Vec<bool> {
    &mut self.selected
  }

  #[inline]
  fn get_int1(&self, row_id: usize, col_id: usize ) -> INT1 {      
    let v : &[INT1] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }  

  #[inline]
  fn get_int2(&self, row_id: usize, col_id: usize ) -> INT2 {      
    let v : &[INT2] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_int4(&self, row_id: usize, col_id: usize ) -> INT4 {      
    let v : &[INT4] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_int8(&self, row_id: usize, col_id: usize ) -> INT8 {      
    let v : &[INT8] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_float4(&self, row_id: usize, col_id: usize ) -> FLOAT4 {      
    let v : &[FLOAT4] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_float8(&self, row_id: usize, col_id: usize ) -> FLOAT8 {      
    let v : &[FLOAT8] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_date(&self, row_id: usize, col_id: usize) -> DATE {      
    let v : &[DATE] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_time(&self, row_id: usize, col_id: usize) -> TIME {      
    let v : &[TIME] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  #[inline]
  fn get_timestamp(&self, row_id: usize, col_id: usize) -> TIMESTAMP {      
    let v : &[TIMESTAMP] = as_array(&self.vectors[col_id]);
    unsafe {
      *v.get_unchecked(row_id)
    }
  }

  fn get_text(&self, row_id: usize, col_id: usize) -> &TEXT {    
    let v : &[TEXT] = as_array(&self.vectors[col_id]);
    unsafe {
      v.get_unchecked(row_id)
    }
  }
}