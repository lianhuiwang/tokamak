#[macro_use]
extern crate common;

use common::session::Session;
use common::types::{I32, F32, Ty};
use common::page::{c_api, Page, ROWBATCH_SIZE};
use common::input::InputSource;
use common::storage::{MemTable, RandomTable};

macro_rules! assert_write_rows {
	($gen:expr, $mem:expr, $num:expr, $total_row:expr) => {
		{
		  let page = $gen.next().unwrap();
		  assert_eq!($num, page.value_count());
		  $mem.write(page).ok().unwrap();
		  assert_eq!($total_row, $mem.row_num());
  	}
	}
}


#[test]
pub fn test_next_once()
{
  let schema = &[I32, F32];

  let session = Session;
  let mut gen = RandomTable::new(&session, schema, 5);
  let mut mem = MemTable::new(&session, schema, &["x","y"]);

  assert_write_rows!(gen, mem, 5, 5);
  assert_write_rows!(gen, mem, 0, 5);
}

#[test]
pub fn test_next_once2()
{
  let schema = &[I32, F32];

  let session = Session;
  let mut gen = RandomTable::new(&session, schema, ROWBATCH_SIZE);
  let mut mem = MemTable::new(&session, schema, &["x","y"]);

  assert_write_rows!(gen, mem, ROWBATCH_SIZE, ROWBATCH_SIZE);
  assert_write_rows!(gen, mem, 0,             ROWBATCH_SIZE);
}


#[test]
pub fn test_next_multiple()
{
  let schema = &[I32, F32];

  let session = Session;
  let mut gen = RandomTable::new(&session, schema, (ROWBATCH_SIZE * 2) + 100);
  let mut mem = MemTable::new(&session, schema, &["x","y"]);

  assert_write_rows!(gen, mem, ROWBATCH_SIZE, ROWBATCH_SIZE);
  assert_write_rows!(gen, mem, ROWBATCH_SIZE, ROWBATCH_SIZE * 2);
  assert_write_rows!(gen, mem, 100,           ROWBATCH_SIZE * 2 + 100);
  assert_write_rows!(gen, mem, 0,             ROWBATCH_SIZE * 2 + 100);
}

#[test]
pub fn test_read()
{
  let schema = &[I32, F32];

  let session = Session;
  let mut gen = RandomTable::new(&session, schema, 5);
  let mut mem = MemTable::new(&session, schema, &["x","y"]);

  assert_write_rows!(gen, mem, 5, 5);
  assert_write_rows!(gen, mem, 0, 5);

  let reader = mem.reader();
  for x in reader {
  	let r: (i32, f32) = x.ok().unwrap();
  }
}


#[test]
pub fn test_write_projected()
{
  let schema = &[I32, F32, I32];

  let session = Session;
  let mut gen = RandomTable::new(&session, schema, 5);
  let mut mem = MemTable::new(&session, &[F32, I32], &["x", "y"]);

  let page      = gen.next().unwrap();
  let projected = page.project(&[1,2]);

  assert_eq!(5, page.value_count());
  assert_eq!(3, page.chunk_num());

  let mut empty_page = Page::empty_page(2);
  empty_page.set_chunks(&projected[..]);
  empty_page.set_value_count(page.value_count());

  let omitted_chunk_sz = page.chunk(0).size;

  assert_eq!(2, empty_page.chunk_num());
  assert_eq!(page.size() - omitted_chunk_sz , empty_page.size());
  assert_eq!(page.value_count(), empty_page.value_count());

  mem.write(&empty_page).ok().unwrap();
  assert_eq!(5, mem.row_num());

  let reader = mem.reader();
  let mut row_id = 0usize;

  unsafe {
    for x in reader {
      let r: (f32, i32) = x.ok().unwrap();
      assert_eq!(c_api::read_f32_raw(page.chunk(1), row_id), r.0);
      assert_eq!(c_api::read_i32_raw(page.chunk(2), row_id), r.1);

      row_id += 1;
    }
  }
}