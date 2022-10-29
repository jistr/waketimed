mod def;
mod error;
mod name;
mod params;
mod value;

pub use def::{BuiltinPollDef, VarDataType, VarDef, VarKind};
pub use error::{VarError, VarNameError};
pub use name::VarName;
pub use params::param_required;
pub use value::VarValue;
