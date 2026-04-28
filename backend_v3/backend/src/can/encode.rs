use super::*;

pub struct CanTxCommand {
    pub priority: Priority,
    pub target: CanNode,
    pub command: CanCommand,
    pub payload: Vec<u8>,
}
impl CanTxCommand {
    pub fn new(priority: Priority, target: CanNode, command: CanCommand, payload: Vec<u8>) -> Self {
        Self {
            priority,
            target,
            command,
            payload,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }
}
