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
pub use error::ProteanError;

// macro_rules! create_patch {
//   // Doing the patch macro here
// }

// TODO: Use this: https://blog.cloudflare.com/writing-complex-macros-in-rust-reverse-polish-notation/
#[macro_export]
/// Bulk apply changes directly to a struct using its setters
///
/// When a struct has private fields, a getter/setter must be used which makes things awkward. In the case
/// of patchwork, we want to generate a diff patch
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
///
/// This keeps an ordered list of Patchwork Patches that have been applied to a struct.
///
/// The two use cases I'm creating this for:
/// - Sending out events based on changes to cached values
/// - Rollback on error based on original values
pub trait Historic<'a, SubClass = Self>: Patchwork<'a> {
  // HACK: Fix language for pop
  // Revert to the previous state and return a patch that can undo the revert
  // fn pop(&mut self) -> Result<Patch>
}

/// A method of creating and detecting mutations between structs
///
/// This is a deeper comparator than the standard Eq/PartialEq, returning a patch listing the differences
/// between two instances of the same type. This is designed to work in the same way unix diff works, and the
/// result is a Patch where
pub trait Patchwork<'a, SubClass = Self>: Debug + Clone + Serialize + Deserialize<'a> {
  fn new_patch(&self) -> Patch {
    // The validator is going to be generated by the macro. If manually implemented, it leaves items open for
    // panic and will be very difficult to debug
    let validator = |_key, _value| {
      // log::debug!("In the Patchwork Validator for 'STRUCT NAME HERE'");
      // log::debug!("key='{:#?}', value='{:#?}'", key, value);

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
  fn apply(&mut self, patch: &Patch) -> Result<()> {
    log::debug!("Applying patch:\n{}", patch);
    // for key in patch.value_map.
    // Split key (recursive calls)

    Ok(())
  }

  /// Compare two structs of the same type and return a Patch needed to convert the left to the right
  fn diff(&self, struct2: &SubClass) -> Result<Patch>;
  // fn get_value(&self, key: Option<&str>) -> SubClass;
  // fn set_value(&self, key: Option<&str>, value: String) -> Result<ProteanError>;
}

/// A container for managing a set of changes to a given implementation of Patchwork
#[derive(Clone)]
pub struct Patch {
  /// The name of the struct that created the patch
  patch_type: String,

  /// A validating closure that ensures that only
  validator: Rc<dyn Fn(String, String) -> Result<()>>,

  /// The map is so we can gather a bulk update.
  ///
  /// The key is the location of the value within the object encoded in dot notation.
  /// THINK: diff of HashMap where the key is not a primitive?
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
  pub fn add(&mut self, key: String, value: String) -> Result<Patch> {
    let validator = &self.validator;
    validator(key.clone(), value.clone())?;
    self.value_map.insert(key, value);
    Ok(self.clone())
  }

  /// Combine two
  pub fn merge(&mut self, prefix: &str, patch: Patch) -> Result<Patch> {
    patch
      .value_map
      .iter()
      .fold(Ok(self.clone()), |acc, (k, v)| {
        // THINK: Does this need to be optimized to get rid of the validator?
        let key = match &k[..] {
          "&self" => prefix.to_string(),
          _ => format!("{}.{}", prefix, k),
        };
        acc?.add(key, v.clone())
      })
  }

  /// Checks to see if the patch has any values stored in it
  pub fn is_empty(&self) -> bool {
    self.value_map.is_empty()
  }
}

//****************************************   Primitive Type Implementations ********************************/
/// Implement all the primitives with a common set of code.
///
/// These are types of values that simple equality works for. String is included, as we are looking at it
/// holistically and not as an array of characters
macro_rules! primitive_patchwork {
  ($type:ty) => {
    impl<'a> Patchwork<'a> for $type {
      /// ```
      /// let i = 10;
      /// let patch = i.diff(10)
      /// ```
      fn diff(&self, struct2: &$type) -> Result<Patch> {
        let mut patch = self.new_patch();
        log::debug!("self: {:#?}, struct2: {:#?}", &self, struct2);
        if self != struct2 {
          patch.add("&self".to_string(), serde_json::to_string(struct2)?)?;
        }
        Ok(patch)
      }
    }
  };
}

// Basic Primitives
primitive_patchwork! {bool}

primitive_patchwork! {i8}
primitive_patchwork! {i16}
primitive_patchwork! {i32}
primitive_patchwork! {i64}
primitive_patchwork! {i128}
primitive_patchwork! {isize}

primitive_patchwork! {u8}
primitive_patchwork! {u16}
primitive_patchwork! {u32}
primitive_patchwork! {u64}
primitive_patchwork! {u128}
primitive_patchwork! {usize}

primitive_patchwork! {f32}
primitive_patchwork! {f64}

primitive_patchwork! {char}
primitive_patchwork! {String}

// TODO: &str

//****************************************   Complex Type Implementations ********************************/
// Complex primitives
// TODO: &T
// TODO: Option
// TODO: Vec
// TODO: HashMap

// Doesn't work because there is no clone() for str
// primitive_patchwork! {str}

/* Serde Example for how it serializes a primitive

  - **Primitive types**:
    - bool
    - i8, i16, i32, i64, i128, isize
    - u8, u16, u32, u64, u128, usize
    - f32, f64
    - char
    - str
    - &T and &mut T


    #[doc(hidden)]
    #[macro_export]
    macro_rules! __private_serialize {
        () => {
            trait Serialize {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: $crate::Serializer;
            }
        };
    }


    /// Serialize an `f64` value.
    ///
    /// ```edition2018
    /// # use serde::Serializer;
    /// #
    /// # serde::__private_serialize!();
    /// #
    /// impl Serialize for f64 {
    ///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    ///     where
    ///         S: Serializer,
    ///     {
    ///         serializer.serialize_f64(*self)
    ///     }
    /// }
    /// ```
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error>;
*/