mod error;
pub mod model;

pub use error::*;
pub type Value = zvariant::Value<'static>;
