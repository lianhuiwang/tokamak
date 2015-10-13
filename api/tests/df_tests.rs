extern crate api;
extern crate common;

use api::TokamakContext;
use api::df::{DataFrame,RandomGenerator};
use common::types::TypeId;

#[test]
pub fn test_data_source() {
  let ctx = TokamakContext::new().ok().unwrap();
  
  let df = ctx.from(RandomGenerator(vec!["int4", "int4"]));
  assert_eq!("from", df.kind());
  
  let selected = df.select(vec![]);
  assert_eq!("select", selected.kind());
  //let rnd: Box<DataSet> = RandomGenerator::new(&ctx, vec!["int4", "int4"]).ok().unwrap();
}