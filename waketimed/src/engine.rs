use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use tokio::sync::mpsc::UnboundedSender;

pub struct Engine {
    _dbus_send: UnboundedSender<DbusMsg>,
    _worker_send: UnboundedSender<WorkerMsg>,
}

impl Engine {
    pub fn new(
        dbus_send: UnboundedSender<DbusMsg>,
        worker_send: UnboundedSender<WorkerMsg>,
    ) -> Self {
        Self {
            _dbus_send: dbus_send,
            _worker_send: worker_send,
        }
    }

    pub fn handle_msg(&mut self, _msg: EngineMsg) {}
}
