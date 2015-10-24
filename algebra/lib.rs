//!
//! Plan
//!

use std::fmt;
use std::result::Result;

use util::tree::{Tree, TreeBuildError};

pub enum AlgebraError 
{
  EmptyStack,
  StillRemainStackItem
}

impl From<TreeBuildError> for AlgebraError {
  fn from(e: TreeBuildError) -> AlgebraError {
    match e {
      TreeBuildError::EmptyStack => AlgebraError::EmptyStack,
      TreeBuildError::StillRemainStackItem => AlgebraError::StillRemainStackItem,
    }
  }
}

pub trait DataSet : fmt::Display
{
  fn id    (&self) -> &str;
  fn kind  (&self) -> &str;
  fn schema(&self) -> &Vec<String>;
  fn uri   (&self) -> Option<&str>;
}

pub struct RegistredFormatData
{
  id: String,
  kind: String,
  schema: Option<Vec<String>>,
  props : Option<Vec<(String, String)>>
}

impl RegistredFormatData
{
  pub fn new(
      id   : &str, 
      kind : &str, 
      types: Option<Vec<&str>>, 
      props: Option<Vec<(&str, &str)>>) -> RegistredFormatData 
  {
    RegistredFormatData {
      id     : id.to_string(),
      kind   : kind.to_string(),
      
      schema : match types {
                 Some(t) => { 
                   Some(t.iter()
                   .map(|s| s.to_string())
                   .collect::<Vec<String>>())
                 },
                 None => None
               },
      
      props  : match props {
                 Some(p) => { 
                   Some(p.iter()
                   .map(|p| (p.0.to_string(), p.1.to_string()))
                   .collect::<Vec<(String, String)>>())
                 },
                 None => None
               }    
    }
  }
}

impl fmt::Display for RegistredFormatData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "id={}, kind={}", self.id, self.kind)
    }
}

impl DataSet for RegistredFormatData 
{
  fn id(&self) -> &str
  {
    &self.id
  }
  
  fn kind(&self) -> &str
  {
    &self.kind
  }
  
  fn schema(&self) -> &Vec<String>
  {
    self.schema.as_ref().unwrap()
  }
  
  fn uri(&self) -> Option<&str>
  {
    None
  }
}

pub enum JoinType
{
  INNER,
  LeftOuter,
  RightOuter,
  FullOuter
}

pub enum Operator 
{
  Scan      (Box<DataSet>),
  Project   (Vec<Operator>),                               // child, exprs
  Filter    (Vec<Operator>),                               // child, bool exprs in a CNF form
  Join      (JoinType, Box<Operator>, Vec<Operator>),      // join type, left, right, join condition
  Aggregate (Vec<Operator>, Vec<Operator>),                // keys, exprs    
  Head      (usize),                                       // row number to fetch
  Tail      (usize),                                       // row number to fetch
}

pub struct AlgebraBuilder 
{
  builder: TreeBuilder<Operator>
}

impl AlgebraBuilder
{
  #[inline]
  pub fn new() -> AlgebraBuilder 
  {
    AlgebraBuilder {
      builder: TreeBuilder::new();
    }
  }
  
  #[inline]
  pub fn build(mut self) -> Result<Operator, AlgebraError>
  {
    try!(self.builder.build())
  }
  
  #[inline]
  fn push(&mut self, op: Operator) -> &mut AlgebraBuilder
  {
    self.builder.push(op);
    self
  }
  
  pub fn dataset(&mut self, dataset: Box<DataSet>) -> &mut AlgebraBuilder 
  {
    self.push(TreeNode::Leaf(Operator::Scan(dataset)));
    self    
  } 
  
  
  #[inline]
  pub fn filter(&mut self, filter: Vec<Operator>) -> &mut AlgebraBuilder 
  {
    self.builder.
    self.push(TreeNode::Branch(op))
  }
  
  pub fn join(
      &mut self, 
      join_type: JoinType,  
      cond: Vec<Operator>) -> &mut AlgebraBuilder 
  {
    debug_assert!(self.stack.len() > 1);
    let left = self.stack.pop().unwrap();
    let right = self.stack.pop().unwrap();
    self.push(Operator::Join(join_type, Box::new(left), Box::new(right), cond));
    self
  }
  
  pub fn join_with(
      &mut self, 
      join_type: JoinType, 
      right: Operator, 
      cond: Vec<Operator>) -> &mut AlgebraBuilder
  {
    debug_assert!(self.stack.len() > 0);
    let left = self.stack.pop().unwrap();
    self.push(Operator::Join(join_type, Box::new(left), Box::new(right), cond));
    self
  }
}

impl fmt::Display for AlgebraBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "stack={}", self.stack.len())
    }
}

/// Visitor for Expr Tree
#[allow(unused_variables)]
pub trait Visitor<'v, T>: Sized {
 
  fn visit_dataset(
      &self, 
      &mut T, 
      dataset: &'v DataSet) {}
 
  fn visit_project(
      &self, 
      context: &mut T, 
      child: &'v Operator, 
      exprs: &Vec<Operator>) {
      
    walk_op(self, context, child);
  }
  
  fn visit_filter(
      &self, 
      context: 
      &mut T, 
      child: &'v Operator, 
      filter: &Vec<Operator>) {
      
    walk_op(self, context, child);
  }

  fn visit_join(
      &self, 
      context: 
      &mut T,
      join_type: &JoinType,
      left: &'v Operator,
      right: &'v Operator, 
      cond: &Vec<Operator>) {
    
    walk_op(self, context, left);  
    walk_op(self, context, right);
  }        

  fn visit_aggregate(
      &self, 
      context: 
      &mut T, 
      child: &'v Operator, 
      keys: &Vec<Operator>,
      exprs:&Vec<Operator>) {
      
    walk_op(self, context, child);
  }      
      
  
  fn visit_head(
      &self, 
      context: &mut T, 
      child: &'v Operator, 
      fetch_row: usize) {
        
    walk_op(self, context, child);
  }
  
  fn visit_tail(
      &self, 
      context: &mut T, 
      child: &'v Operator, 
      fetch_row: usize) {
        
    walk_op(self, context, child);
  }
}

/// Walker for Expr Tree
pub fn walk_op<'v, T, V>(v: &V, ctx: &mut T, op: &'v Operator) 
    where V: Visitor<'v, T> {
  match *op {
    Operator::Scan     (ref ds)                        => { v.visit_dataset(ctx, &**ds)},
    Operator::Project  (ref child,ref exprs)           => { v.visit_project(ctx, &**child, exprs) },
    Operator::Filter   (ref child,ref filters)         => { v.visit_filter(ctx, &**child, filters) },
    Operator::Aggregate(ref child,ref keys, ref exprs) => { v.visit_aggregate(ctx, &**child, keys, exprs) },
    Operator::Join     (ref join_type, ref left, ref right, ref cond) => {  v.visit_join(ctx, join_type, &**left, &**right, cond) },
    Operator::Head     (ref child,num)                 => { v.visit_head(ctx, &**child,num) },
    Operator::Tail     (ref child,num)                => { v.visit_tail(ctx, &**child,num) },
  }
}
 
    
   