use wtd_core::vars::{VarDef, VarName, VarValue};

#[derive(Debug, PartialEq)]
pub enum EngineMsg {
    PollVarsTick,
    ReturnVarPoll(VarName, Option<VarValue>),
    Terminate,
}

#[derive(Debug, PartialEq)]
pub enum WorkerMsg {
    CallVarPoll(VarName),
    LoadPollVarFns(VarDef),
    // SpawnPollVarInterval(ms)
    SpawnPollVarInterval(u64),
    // Suspend(test_mode)
    Suspend(bool),
    Terminate,
}
