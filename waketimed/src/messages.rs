use crate::var_fns::OptVarValueFuture;
use std::fmt;
use wtd_core::vars::{VarName, VarValue};

#[derive(Debug, PartialEq)]
pub enum EngineMsg {
    PollVarsTick,
    ReturnVarPoll(VarName, Option<VarValue>),
    Terminate,
}

pub enum WorkerMsg {
    CallVarPoll(
        VarName,
        Box<dyn FnOnce() -> OptVarValueFuture + Send + Sync>,
    ),
    SpawnPollVarInterval(u64),
    Terminate,
}

impl fmt::Debug for WorkerMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        use WorkerMsg::*;
        write!(f, "WorkerMsg::")?;
        match self {
            CallVarPoll(ref var_name, _) => write!(f, "CallVarPoll({:?}, _)", var_name),
            SpawnPollVarInterval(interval) => write!(f, "SpawnPollVarInterval({})", interval),
            Terminate => write!(f, "Terminate"),
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
