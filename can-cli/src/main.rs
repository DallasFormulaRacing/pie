#[cfg(target_os = "linux")]
#[allow(dead_code)]
#[path = "../../backend_v3/backend/src/can/mod.rs"]
mod can;

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write as _};
    use std::time::{Duration, Instant};

    let interface = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("CAN_INTERFACE").ok())
        .unwrap_or_else(|| "can0".to_string());

    let socket = can::socket::CanSocket::open(&interface)?;
    socket.set_read_timeout(Duration::from_millis(100))?;

    println!("opened {interface}");
    println!("commands: i=imu, w=wheel speed, bt=brake temp, tt=tire temp, q=quit");

    loop {
        print!("can-cli> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            break;
        }

        let input = input.trim().to_lowercase();
        if matches!(input.as_str(), "q" | "quit" | "exit") {
            break;
        }

        let Some(request) = DaqRequest::from_input(&input) else {
            eprintln!("unknown command: {input}. use i, w, bt, tt, or q");
            continue;
        };

        let message = request.to_can_message();
        socket.write_message(&message)?;
        println!(
            "sent {:?} to {:?} as id=0x{:08X}",
            request.command,
            message.id.target,
            message.id.to_raw_id()
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut received_expected_response = false;

        while Instant::now() < deadline {
            match socket.try_read_message() {
                Ok(Some(message)) => {
                    print_message(&message);
                    if request.matches_response(&message) {
                        received_expected_response = true;
                        break;
                    }
                }
                Ok(None) => {}
                Err(error) => eprintln!("CAN socket read error: {error}"),
            }
        }

        if !received_expected_response {
            eprintln!("timed out waiting for {:?}", request.response_command);
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("can-cli requires Linux SocketCAN support to read a CAN interface".into())
}

#[cfg(target_os = "linux")]
fn print_message(message: &can::DfrCanMessageBuf) {
    let raw_id = message.id.to_raw_id();
    let command = u16::from(message.id.command);
    let data = format_data(&message.data);

    println!(
        "id=0x{raw_id:08X} priority={} target={:?} source={:?} command={:?} command_raw=0x{command:04X} len={} data=[{}]",
        message.id.priority,
        message.id.target,
        message.id.source,
        message.id.command,
        message.data.len(),
        data,
    );
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
struct DaqRequest {
    command: can::DaqCanCommand,
    response_command: can::DaqCanCommand,
}

#[cfg(target_os = "linux")]
impl DaqRequest {
    fn from_input(input: &str) -> Option<Self> {
        let (command, response_command) = match input {
            "i" | "imu" => (can::DaqCanCommand::ReqImuData, can::DaqCanCommand::ImuData),
            "w" | "ws" | "wheel" | "wheelspeed" | "wheel-speed" => (
                can::DaqCanCommand::ReqSpeedData,
                can::DaqCanCommand::SpeedData,
            ),
            "bt" | "brake" | "braketemp" | "brake-temp" => (
                can::DaqCanCommand::ReqTempData,
                can::DaqCanCommand::TempData,
            ),
            "tt" | "tire" | "tiretemp" | "tire-temp" => (
                can::DaqCanCommand::ReqTempData,
                can::DaqCanCommand::TempData,
            ),
            _ => return None,
        };

        Some(Self {
            command,
            response_command,
        })
    }

    fn to_can_message(self) -> can::DfrCanMessageBuf {
        can::DfrCanMessageBuf {
            id: can::DfrCanId {
                priority: 1,
                target: can::CanNode::Nucleo1,
                source: can::CanNode::Raspi,
                command: can::CanCommand::Daq(self.command),
            },
            data: Vec::new(),
        }
    }

    fn matches_response(self, message: &can::DfrCanMessageBuf) -> bool {
        message.id.source == can::CanNode::Nucleo1
            && message.id.target == can::CanNode::Raspi
            && message.id.command == can::CanCommand::Daq(self.response_command)
    }
}

#[cfg(target_os = "linux")]
fn format_data(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
