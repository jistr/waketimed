use crate::core::vars::{VarDef, VarName, VarValue};

#[derive(Debug, PartialEq, Eq)]
pub enum EngineMsg {
    PollVarsTick,
    ReturnVarPoll(VarName, Option<VarValue>),
    SystemIsResuming,
    SystemIsSuspending,
    Terminate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkerMsg {
    CallVarPoll(VarName),
    LoadPollVarFns(VarDef),
    // SpawnPollVarInterval(ms)
    SpawnPollVarInterval(u64),
    // Suspend(test_mode)
    Suspend(bool),
    Terminate,
    WatchPrepareForSleep,
}
