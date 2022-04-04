#[derive(Debug, PartialEq, Eq)]
pub enum DbusMsg {
    Terminate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EngineMsg {
    Terminate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkerMsg {
    Placeholder,
}
