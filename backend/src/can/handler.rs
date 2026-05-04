use super::{BmsData, CanData, CanDataError, CanSystem, DaqData, DfrCanMessage};

pub fn data_for_message(message: &DfrCanMessage) -> Result<Option<CanData>, CanDataError> {
    match message.id.source.system() {
        Some(CanSystem::Daq) => handle_daq_message(message),
        Some(CanSystem::Bms) => handle_bms_message(message),
        Some(CanSystem::Vcu) | None => Ok(None),
    }
}

fn handle_daq_message(message: &DfrCanMessage) -> Result<Option<CanData>, CanDataError> {
    match DaqData::try_from(message) {
        Ok(data) => Ok(Some(CanData::Daq(data))),
        Err(CanDataError::UnsupportedCommand(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

fn handle_bms_message(message: &DfrCanMessage) -> Result<Option<CanData>, CanDataError> {
    match BmsData::try_from(message) {
        Ok(data) => Ok(Some(CanData::Bms(data))),
        Err(CanDataError::UnsupportedCommand(_) | CanDataError::UnsupportedBmsPayload(_)) => {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::can::{
        BmsCanCommand, CanCommand, CanNode, CommonCanCommand, DaqCanCommand, DfrCanId,
    };

    fn message(source: CanNode, command: CanCommand, data: Vec<u8>) -> DfrCanMessage {
        DfrCanMessage {
            id: DfrCanId {
                priority: 1,
                target: CanNode::Raspi,
                source,
                command,
            },
            data,
        }
    }

    #[test]
    fn daq_source_dispatches_to_daq_handler() {
        let event = data_for_message(&message(
            CanNode::FrontLeft,
            CanCommand::Daq(DaqCanCommand::TempData),
            vec![0; 64],
        ))
        .expect("DAQ payload should decode")
        .expect("DAQ data should be routed");

        assert!(matches!(event, CanData::Daq(DaqData::Temperature { .. })));
    }

    #[test]
    fn bms_source_dispatches_to_bms_handler_without_fake_telemetry() {
        let data = data_for_message(&message(
            CanNode::Bms,
            CanCommand::Bms(BmsCanCommand::BatteryPackData),
            vec![0; 8],
        ))
        .expect("unsupported BMS layouts should not be fatal");

        assert!(data.is_none());
    }

    #[test]
    fn unsupported_sources_do_not_emit_telemetry() {
        for source in [CanNode::Raspi, CanNode::AllNodes, CanNode::Vcu] {
            let data = data_for_message(&message(
                source,
                CanCommand::Common(CommonCanCommand::Ping),
                Vec::new(),
            ))
            .expect("unsupported source should not be fatal");

            assert!(data.is_none());
        }
    }
}
