//! Converting Phases
//!
//! # Simple Version
//!
//! A DataFrame dag --(ExecutorPlanner)--> A ExecutionPlan --(Parallelizer)-->
//! A set of Tasks --(TaskRunner)--> DataSet

extern crate algebra;
extern crate common;
extern crate default_package;
extern crate exec;
extern crate optimizer;
extern crate plan;
extern crate storage;

mod runner;
pub use runner::QueryRunner;

mod local_runner;
pub use local_runner::LocalQueryRunner;
