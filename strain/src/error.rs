//! All the errors that can be returned from Strain

use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[cfg_attr(
  feature = "serde_support",
  derive(serde::Serialize, serde::Deserialize)
)]
pub enum StrainError {
  #[error("There was an error attempting to convert from one type to another")]
  ConversionError,
}
