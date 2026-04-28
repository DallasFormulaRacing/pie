use super::*;
pub struct CanEnvelope<'a> {
    pub priority: Priority,
    pub target: CanNode,
    pub source: CanNode,
    pub command: CanCommand,
    pub data: &'a [u8],
}

impl<'a> CanEnvelope<'a> {
    pub fn new(
        priority: Priority,
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
