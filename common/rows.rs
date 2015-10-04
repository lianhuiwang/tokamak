/// This row implementation is basically based on Pax, 
/// but it has a variation in terms of variable-length blocks.
///
/// ## Deign Consideration
///
/// The API is being designed with the following design consideration:
/// 
/// * Reuseable allocated memory
/// * Vectorized processing
/// * Various encodings (light/heavy weight compression and no 
///   deserialization from storage pages)
/// * Late materialization (Refer to [3])
///
/// ## References
/// * [1] Daniel Abadi et al., The Design and Implementation of Modern Column-Oriented Database 
///       Systems
/// * [2] 
/// * [3] Daniel J. Abadi ea al., Materialization Strategies in a Column-Oriented DBMS, ICDE 2007

use std::marker;
use std::rc::Rc;

use types::Type;
use platform::{CACHE_LINE_SIZE, get_aligned_size};

/// Each executor and operator process a batch of rows at a time for better throughput.
/// The experiment of MonetDB presented that 1024 is the best number of a row batch.
/// It's reason why I currently use 1024 as the number of row batch. 
pub static ROWBATCH_SIZE: usize = 1024; 

/// Type for column index
pub type PageId = usize;
/// Type for row position
pub type PosId = usize;

pub struct Page 
{
  mini_pages: Vec<Box<MiniPage>>
}

impl Page 
{
  #[inline]
  pub fn minipage_num(&self) -> usize { self.mini_pages.len() }
  
  #[inline]
  pub fn minipage(&self, id: PageId) -> &MiniPage 
  {
    debug_assert!(id < self.minipage_num());
     
    &*self.mini_pages[id] 
  }
  
  /// Total byte size of this page
  #[inline]
  pub fn bytesize(&self) -> u32 
  {
    self.mini_pages.iter()
      .map(|m| m.bytesize())
      .fold(0, |acc, size| acc + size)
  }
}

pub trait MiniPage 
{
  fn bytesize(&self) -> u32;
  
  fn read_i8(&self, pos: PosId) -> i8;
  
  fn read_i16(&self, pos: PosId) -> i16;
  
  fn read_i32(&self, pos: PosId) -> i32;
  
  fn read_i64(&self, pos: PosId) -> i64;
  
  fn read_f32(&self, pos: PosId) -> f32;
  
  fn read_f64(&self, pos: PosId) -> f64;
  
  fn writer(&mut self) -> &mut MiniPageWriter;
}

/// Writer for Vector. The writer internally must have a cursor to write a value.
/// For each write, the cursor must move forward the cursor.   
/// You must call finalize() before reading any value from the Vector.  
pub trait MiniPageWriter 
{
  fn write_i8(&mut self, v: i8);
  
  fn write_i16(&mut self, v: i16);
  
  fn write_i32(&mut self, v: i32);
  
  fn write_i64(&mut self, v: i64);
  
  fn write_f32(&mut self, v: f32);
  
  fn write_f64(&mut self, v: f64);
  
  fn write_bytes(&mut self, v: &[u8]);
  
  fn reset(&mut self);
  
  fn finalize(&mut self);
}


pub struct PageBuilder 
{
  page: Page
}

impl PageBuilder 
{
  pub fn new(types: &Vec<Box<Type>>) -> Self 
  {
    let mini_pages = types
      .iter()
      .map(|ty| ty.handler())
      .map(|f| (f.create_minipage)())
      .collect::<Vec<Box<MiniPage>>>();
    
    PageBuilder {page: Page {mini_pages: mini_pages}}
  }
  
  pub fn writer(&mut self, id: PageId) -> &mut MiniPageWriter 
  {
    self.page.mini_pages[id].writer()
  }
  
  pub fn build(&mut self) -> &Page 
  {
    for v in self.page.mini_pages.iter_mut() {
      v.writer().finalize();
    }
    
    &self.page
  }
}