use crate::messages::{DbusMsg, EngineMsg, WorkerMsg};
use tokio::sync::mpsc::UnboundedSender;

pub struct Engine {
    dbus_send: UnboundedSender<DbusMsg>,
    worker_send: UnboundedSender<WorkerMsg>,
}

impl Engine {
    pub fn new(
        dbus_send: UnboundedSender<DbusMsg>,
        worker_send: UnboundedSender<WorkerMsg>,
    ) -> Self {
        Self {
            dbus_send,
            worker_send,
        }
    }

    pub fn handle_msg(&mut self, msg: EngineMsg) {
        match msg {
            EngineMsg::Terminate => self.handle_terminate(),
        }
    }

    fn handle_terminate(&mut self) {
        self.dbus_send
            .send(DbusMsg::Terminate)
            .expect("Failed to send DbusMsg::Terminate");
        self.worker_send
            .send(WorkerMsg::Terminate)
            .expect("Failed to send WorkerMsg::Terminate");
    }
}
