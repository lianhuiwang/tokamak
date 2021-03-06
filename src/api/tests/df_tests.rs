extern crate api;
extern crate common;
extern crate env_logger;
#[macro_use] extern crate log;

extern crate rustc_serialize;
extern crate bincode;

use bincode::rustc_serialize::{encode, decode};
use rustc_serialize::Decodable;


use api::TokamakContext;
use api::df::DataFrame;

#[test]
pub fn test_data_source() {
	env_logger::init().unwrap();
	
  let ctx = TokamakContext::new().ok().unwrap();
  
  let df = ctx.random_table(vec!["i32", "f32"], 5);
  assert_eq!("from", df.kind());
  
  let selected = df.select(vec![]);
  assert_eq!("select", selected.kind());
  
  selected.head();
}

#[test]
pub fn test_head() {
  let ctx = TokamakContext::new().ok().unwrap();
  let df = ctx.random_table(vec!["i32", "f32"], 5);
  //println!("{}", df.head().ok().unwrap());
}

struct Record<D: Decodable> {
  _phantom: ::std::marker::PhantomData<D>,
}

pub struct Xxx;
impl Xxx {
  fn decode<'a, D: Decodable>(&'a mut self) -> Option<Record<D>> {
    None
  }
}

#[test]
pub fn test_tuple() {
  let mut x = Xxx;
  
  let tuple: Option<Record<(i32, i32, String)>> = x.decode();
}

//#[test]
pub fn test_bincode() {
  // The object that we will serialize.
    let target = Some("hello world".to_string());
    // The maximum size of the encoded message.
    let limit = bincode::SizeLimit::Bounded(20);

    let encoded: Vec<u8>        = encode(&target, limit).unwrap();
    let decoded: Option<(String, i32)> = decode(&encoded[..]).unwrap();
    println!("{}", decoded.unwrap().0);
    //assert_eq!(target, decoded);
}