//!
//! Plugins
//!

use std::collections::HashMap;

use super::err::{TResult, void_ok, Void};
use super::func::FuncRegistry;
use super::types::TypeRegistry;

pub struct PackageManager {
  pkgs    : HashMap<String, Box<Package>>,
  type_reg: TypeRegistry,
  func_reg: FuncRegistry  
}

impl PackageManager {
  pub fn new() -> PackageManager {
    PackageManager {
      pkgs: HashMap::new(),
      type_reg: TypeRegistry::new(),
      func_reg: FuncRegistry::new(),
    }
  }
  
  pub fn new_with(pkgs: Vec<Box<Package>>) -> PackageManager {
    PackageManager {
      pkgs: pkgs.into_iter()
              .map(|p: Box<Package>| -> (String, Box<Package>) { 
                (p.name().to_string(), p) 
              })
              .collect::<HashMap<String, Box<Package>>>(),
      type_reg: TypeRegistry::new(),
      func_reg: FuncRegistry::new(),
    }
  }
  
  pub fn add(&mut self, pkg: Box<Package>) -> &mut Self {
    // TODO - To ensure add packages before loading, we need to adopt Builder pattern. 
    self.pkgs.insert(pkg.name().to_string(), pkg);
    self
  }
  
  pub fn load_all(&mut self) -> Void {
    for (name, pkg) in self.pkgs.iter_mut() {
      try!(pkg.load(&mut self.type_reg, &mut self.func_reg))            
    }
    
    Ok(())
  }
  
  pub fn ty_registry(&self) -> &TypeRegistry {
    &self.type_reg    
  }
  
  pub fn fnc_registry(&self) -> &FuncRegistry {
    &self.func_reg
  }
}

pub trait Package {
  fn name(&self) -> &str;
  fn load(&mut self, type_reg: &mut TypeRegistry, fn_reg: &mut FuncRegistry) -> Void;
}