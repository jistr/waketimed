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
    SpawnPollVarInterval(u64),
    Terminate,
}
