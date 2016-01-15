#[macro_use] extern crate common;

use common::session::Session;
use common::types::{F32, I32};
use common::page::{c_api, Page, MiniPage, ROWBATCH_SIZE};
use common::input::InputSource;
use common::storage::RandomTable;

#[test]
pub fn test_minipage_copy()
{
  let session = Session;
  let mut generator = RandomTable::new(&session, &[I32, F32], ROWBATCH_SIZE);

  {
		let page = generator.next().unwrap();
		let m1: &MiniPage = page.minipage_ref(0);
		let m2: &MiniPage = page.minipage_ref(1);

	  let m1_copy = m1.copy();
	  let m2_copy = m2.copy();

    unsafe {
      for x in 0 .. ROWBATCH_SIZE {
        assert_eq!(c_api::read_raw_i32(m1, x), c_api::read_raw_i32(&m1_copy, x));
        assert_eq!(c_api::read_raw_i32(m2, x), c_api::read_raw_i32(&m2_copy, x));
      }
    }
  }

  generator.close().ok().unwrap();
}

/*
#[test]
pub fn test_project()
{
  let session = Session;
  let mut gen = RandomTable::new(&session, &[I32, F32, F32], 5);

  let page      = gen.next().unwrap();
  let projected = page.project(&vec![1,2]);

  assert_eq!(5, page.value_count());
  assert_eq!(5, projected.value_count());

  assert_eq!(3, page.minipage_num());
  assert_eq!(2, projected.minipage_num());

  unsafe {
    for x in 0 .. 5 {
      assert_eq!(c_api::read_raw_f32(page.minipage(1), x), c_api::read_raw_f32(projected[0], x));
      assert_eq!(c_api::read_raw_i32(page.minipage(2), x), c_api::read_raw_i32(projected[1], x));
    }
  }
}

#[test]
pub fn test_page_copy()
{
  let session = Session;
  let mut generator = RandomTable::new(&session, &[I32, F32], ROWBATCH_SIZE);

  {
		let page = generator.next().unwrap();
		let copied_page = page.to_owned();

		let m1 = page.minipage(0);
		let m2 = page.minipage(1);

	  let m1_copy = copied_page.minipage(0);
	  let m2_copy = copied_page.minipage(1);

    unsafe {
      for x in 0 .. ROWBATCH_SIZE {
        assert_eq!(c_api::read_raw_i32(m1, x), c_api::read_raw_i32(m1_copy, x));
        assert_eq!(c_api::read_raw_i32(m2, x), c_api::read_raw_i32(m2_copy, x));
      }
    }
  }

  generator.close().ok().unwrap();
}
*/