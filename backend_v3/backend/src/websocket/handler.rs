use super::{BmsMessage, DaqMessage, Message, VcuMessage};

pub fn handle_ws_message(message: &Message) {
    match message {
        Message::Daq(message) => handle_daq_message(message),
        Message::Bms(message) => handle_bms_message(message),
        Message::Vcu(message) => handle_vcu_message(message),
    }
}

fn handle_daq_message(message: &DaqMessage) {
    match message {
        DaqMessage::Ping { source } => {
            println!("client -> pi daq ping node {source:?}");
        }
        DaqMessage::Reset { source } => {
            println!("client -> pi: daq reset node {source:?}");
        }
        _ => {
            println!(
                "Incompatible DAQ message from client (client can send command messages only, not telemetry)"
            );
        }
    }
}

fn handle_bms_message(message: &BmsMessage) {
    match message {
        BmsMessage::Reset { source } => {
            println!("device -> pi bms reset from {source:?}");
        }
        BmsMessage::Ping { source } => {
            println!("device -> pi bms ping from {source:?}");
        }
        _ => {
            println!(
                "Incompatible BMS message from client (client can send command messages only, not telemetry)"
            );
        }
    }
}

fn handle_vcu_message(message: &VcuMessage) {
    match message {
        VcuMessage::Reset { source } => {
            println!("device -> pi vcu reset from {source:?}");
        }
        VcuMessage::Ping { source } => {
            println!("device -> pi vcu ping from {source:?}");
        }
        _ => {
            println!(
                "Incompatible VCU message from client (client can send command messages only, not telemetry)"
            );
        }
    }
}
