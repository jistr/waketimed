use std::fmt;
use wtd_core::vars::VarName;

#[derive(Debug, PartialEq)]
pub enum DbusMsg {
    Terminate,
}

#[derive(Debug, PartialEq)]
pub enum EngineMsg {
    ReturnVarIsActive(VarName, bool),
    Terminate,
}

pub enum WorkerMsg {
    CallVarIsActive(VarName, Box<dyn FnOnce() -> bool + Send + Sync>),
    Terminate,
}

impl fmt::Debug for WorkerMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use WorkerMsg::*;
        write!(f, "WorkerMsg::")?;
        match self {
            Terminate => write!(f, "Terminate"),
            CallVarIsActive(ref var_name, _) => write!(f, "CallVarIsActive({:?}, _)", var_name),
        }
    }
}

// PartialEq cannot be derived for WorkerMsg because of using Boxed
// FnOnce in messages. We could implement it like shown below, but
// then we'd have to make sure to edit this every time we add a new
// message definition. Rather we just see if we can go ahead with just
// destructuring instead of using ==.
//
// impl PartialEq for WorkerMsg {
//     fn eq(&self, rhs: &Self) -> bool {
//         use WorkerMsg::*;
//         match (self, rhs) {
//             (Terminate, Terminate) => true,
//             ... other variants which can be considered equal ...
//             (_, _) => false,
//         }
//     }
// }
