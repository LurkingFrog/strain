//! Traits applying and tracking struct mutations
//!
//! This set of tools is to remove the boilerplate when equality is not enough, but we want to know
//! what is different between two structs. As an outgrowth of this, keeping a historical record of the
//! mutations can help in transactional applications, able to roll back changes to a prior state as well
//! as generate better error logs.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub mod error;
pub use error::StrainError;

// macro_rules! create_patch {
//   // Doing the patch macro here
// }

// TODO: Use this: https://blog.cloudflare.com/writing-complex-macros-in-rust-reverse-polish-notation/
#[macro_export]
macro_rules! patch {
  // (Patchwork, One or more k/v pairs)
  ($a:expr, $(($update:expr)),+) => {{
    let mut patch = $a.new_patch();
    $(
      let (key, value) = $update;
      patch.add(key.to_string(), serde_json::to_string(&value).unwrap());
    )*;
    patch
  }};
}

/// Keeps an internal record of mutations to the struct
pub trait Historic<'a, SubClass = Self>: Patchwork<'a> {}

/// A method of creating and detecting mutations between structs
pub trait Patchwork<'a, SubClass = Self>: Debug + Clone + Serialize + Deserialize<'a> {
  fn new_patch(&self) -> Patch {
    // This is going to be generated by the macro. If manually implemented, it leaves items open for panic
    let validator = |key, value| {
      log::debug!("In the Patchwork Validator for 'STRUCT NAME HERE'");
      log::debug!("key='{:#?}', value='{:#?}'", key, value);

      // TODO: Validate key path
      // TODO: Validate value is correct

      Ok(())
    };

    Patch {
      patch_type: "STRUCT NAME HERE".to_string(),
      validator: Rc::new(validator),
      value_map: HashMap::new(),
    }
  }
  fn apply(&mut self, patch: Patch) -> Result<()> {
    log::debug!("Applying patch:\n{}", patch);
    // for key in patch.value_map.
    // Split key (recursive calls)

    Ok(())
  }
  // fn diff(struct1: SubClass, struct2: SubClass) -> Result<Patch>;
  // fn get_value(&self, key: Option<&str>) -> SubClass;
  // fn set_value(&self, key: Option<&str>, value: String) -> Result<StrainError>;
}

/// A container for managing a set of changes to a given implementation of Patchwork
#[derive(Clone)]
pub struct Patch {
  /// The name of the struct that created the patch
  patch_type: String,

  /// A validating closure that ensures that only
  validator: Rc<dyn Fn(String, String) -> Result<()>>,

  /// The map is so we can gather a bulk update
  value_map: HashMap<String, String>,
}

impl std::fmt::Display for Patch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:#?}", self)
  }
}

impl std::fmt::Debug for Patch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Patch<{}>: {:#?} ", self.patch_type, self.value_map)
  }
}

impl Patch {
  /// Add a new record to the patch
  pub fn add(&mut self, key: String, value: String) -> Result<()> {
    let validator = &self.validator;
    validator(key.clone(), value.clone())?;
    self.value_map.insert(key, value);
    Ok(())
  }
}

//****************************************   Primitive Implementations ********************************/
impl<'a> Patchwork<'a> for i32 {
  // fn diff(struct1: i32, struct2: i32) -> Result<Patch> {
  //   unimplemented!("'diff' is not implemented yet")
  // }
}
