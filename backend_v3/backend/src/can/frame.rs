use super::*;
pub struct CanEnvelope<'a> {
    pub priority: u8,
    pub target: CanNode,
    pub source: CanNode,
    pub command: CanCommand,
    pub data: &'a [u8],
}

impl<'a> CanEnvelope<'a> {
    pub fn new(
        priority: u8,
        target: CanNode,
        source: CanNode,
        command: CanCommand,
        data: &'a [u8],
    ) -> Self {
        Self {
            priority,
            target,
            source,
            command,
            data,
        }
    }
}

impl<'a> From<(DfrCanId, &'a [u8])> for CanEnvelope<'a> {
    fn from((id, data): (DfrCanId, &'a [u8])) -> Self {
        Self {
            priority: id.priority,
            target: CanNode::from(id.target),
            source: CanNode::from(id.source),
            command: CanCommand::from(id.command),
            data,
        }
    }
}
