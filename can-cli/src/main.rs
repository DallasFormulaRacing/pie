use std::collections::{HashMap, VecDeque};
use std::io;
use std::time::{Duration, Instant};

use chrono::Local;
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, List, ListItem, Paragraph},
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::connect_async;

const MAX_FEED_LINES: usize = 500;
const MAX_SERIES_POINTS: usize = 300;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = String::from("ws://pi.local:9002");

    let (tx, mut rx) = mpsc::unbounded_channel();
    spawn_input_task(tx.clone());
    tokio::spawn(websocket_task(url.clone(), tx));

    let mut terminal = setup_terminal()?;
    let _terminal_guard = TerminalGuard;
    let mut app = App::new(url);
    let mut redraw_interval = time::interval(Duration::from_millis(50));

    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                if app.handle_event(event) {
                    break;
                }
            }
            _ = redraw_interval.tick() => {}
        }

        terminal.draw(|frame| draw(frame, &app))?;
    }

    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn spawn_input_task(tx: mpsc::UnboundedSender<AppEvent>) {
    std::thread::spawn(move || {
        loop {
            match event::read() {
                Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                    if tx.send(AppEvent::Key(key)).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    let _ = tx.send(AppEvent::Status(format!("terminal input error: {error}")));
                    break;
                }
            }
        }
    });
}

async fn websocket_task(url: String, tx: mpsc::UnboundedSender<AppEvent>) {
    loop {
        let _ = tx.send(AppEvent::Status(format!("connecting to {url}")));

        match connect_async(&url).await {
            Ok((stream, _)) => {
                let _ = tx.send(AppEvent::Connected(true));
                let _ = tx.send(AppEvent::Status(format!("connected to {url}")));
                let (_, mut read) = stream.split();

                while let Some(message) = read.next().await {
                    match message {
                        Ok(message) if message.is_text() => match message.to_text() {
                            Ok(text) => match serde_json::from_str::<BackendEvent>(text) {
                                Ok(event) => {
                                    if tx.send(AppEvent::Backend(event)).is_err() {
                                        return;
                                    }
                                }
                                Err(error) => {
                                    let _ = tx.send(AppEvent::Status(format!(
                                        "JSON parse error: {error}"
                                    )));
                                }
                            },
                            Err(error) => {
                                let _ = tx.send(AppEvent::Status(format!(
                                    "websocket text error: {error}"
                                )));
                            }
                        },
                        Ok(message) if message.is_close() => break,
                        Ok(_) => {}
                        Err(error) => {
                            let _ = tx.send(AppEvent::Status(format!("websocket error: {error}")));
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                let _ = tx.send(AppEvent::Status(format!("connect error: {error}")));
            }
        }

        let _ = tx.send(AppEvent::Connected(false));
        time::sleep(Duration::from_secs(2)).await;
    }
}

#[derive(Debug)]
enum AppEvent {
    Backend(BackendEvent),
    Connected(bool),
    Key(KeyEvent),
    Status(String),
}

struct App {
    url: String,
    connected: bool,
    status: String,
    feed: VecDeque<FeedLine>,
    devices: Vec<Device>,
    selected_device: usize,
    selected_sensor: usize,
    series: HashMap<SeriesKey, VecDeque<(f64, f64)>>,
    sample_index: f64,
    started_at: Instant,
}

impl App {
    fn new(url: String) -> Self {
        Self {
            url,
            connected: false,
            status: "starting".to_string(),
            feed: VecDeque::new(),
            devices: Vec::new(),
            selected_device: 0,
            selected_sensor: 0,
            series: HashMap::new(),
            sample_index: 0.0,
            started_at: Instant::now(),
        }
    }

    fn handle_event(&mut self, event: AppEvent) -> bool {
        match event {
            AppEvent::Backend(event) => self.handle_backend_event(event),
            AppEvent::Connected(connected) => self.connected = connected,
            AppEvent::Status(status) => self.push_status(status),
            AppEvent::Key(key) => return self.handle_key(key),
        }

        false
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Left | KeyCode::Char('h') => self.previous_device(),
            KeyCode::Right | KeyCode::Char('l') => self.next_device(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_sensor(),
            KeyCode::Down | KeyCode::Char('j') => self.next_sensor(),
            KeyCode::Char('c') => self.clear_selected_series(),
            _ => {}
        }

        false
    }

    fn handle_backend_event(&mut self, event: BackendEvent) {
        match event.data {
            BackendEventData::DeviceRegistrySnapshot { devices } => {
                for device in devices {
                    self.push_feed(
                        device.device(),
                        "DEVICE_REGISTRY",
                        format!(
                            "{} {}",
                            device.name,
                            if device.online { "online" } else { "offline" }
                        ),
                    );
                }
            }
            BackendEventData::DeviceStatusChanged { device } => {
                self.push_feed(
                    device.device(),
                    "DEVICE_STATUS",
                    format!(
                        "{} {}",
                        device.name,
                        if device.online { "online" } else { "offline" }
                    ),
                );
            }
            BackendEventData::DaqTelemetry { telemetry } => self.handle_daq_telemetry(telemetry),
            BackendEventData::BmsTelemetry { telemetry } => self.handle_bms_telemetry(telemetry),
            BackendEventData::BackendError { message } => {
                self.push_feed(Device::Raspi, "BACKEND_ERROR", message);
            }
        }
    }

    fn handle_daq_telemetry(&mut self, telemetry: DaqTelemetry) {
        match telemetry {
            DaqTelemetry::Imu { source, samples } => {
                self.select_first_telemetry_source(source);
                self.push_feed(source, "CMD_IMU_DATA", String::new());
                self.ensure_device(source);

                for sample in samples {
                    let x = self.next_x();
                    self.push_point(source, Sensor::AccelX, x, sample.acceleration.x);
                    self.push_point(source, Sensor::AccelY, x, sample.acceleration.y);
                    self.push_point(source, Sensor::AccelZ, x, sample.acceleration.z);
                    self.push_point(source, Sensor::GyroRho, x, sample.angular_acceleration.rho);
                    self.push_point(
                        source,
                        Sensor::GyroTheta,
                        x,
                        sample.angular_acceleration.theta,
                    );
                    self.push_point(source, Sensor::GyroPhi, x, sample.angular_acceleration.phi);
                }
            }
            DaqTelemetry::Temperature { source, samples } => {
                self.select_first_telemetry_source(source);
                self.push_feed(source, "CMD_TEMP_DATA", String::new());
                self.ensure_device(source);
                let tire_average = average(samples.iter().map(|sample| sample.tire));
                let brake_average = average(samples.iter().map(|sample| sample.brake));
                let x = self.next_x();
                self.push_point(source, Sensor::TireAverage, x, tire_average);
                self.push_point(source, Sensor::BrakeAverage, x, brake_average);
            }
            DaqTelemetry::Tbd { source, value } => {
                self.push_feed(source, "DAQ_TBD", format!("{value:.2}"));
                self.ensure_device(source);
            }
        }
    }

    fn handle_bms_telemetry(&mut self, telemetry: BmsTelemetry) {
        match telemetry {
            BmsTelemetry::Voltages { source, readings } => {
                self.select_first_telemetry_source(source);
                self.push_feed(source, "BMS_VOLTAGES", String::new());
                self.ensure_device(source);
                let x = self.next_x();
                self.push_point(source, Sensor::BmsPackVoltage, x, readings.pack);
            }
            BmsTelemetry::Temperatures { source, readings } => {
                self.select_first_telemetry_source(source);
                self.push_feed(source, "BMS_TEMPERATURES", String::new());
                self.ensure_device(source);
                let x = self.next_x();
                self.push_point(source, Sensor::BmsAverageTemp, x, readings.average);
            }
            BmsTelemetry::Balancing { source, .. } => {
                self.push_feed(source, "BMS_BALANCING", String::new());
                self.ensure_device(source);
            }
            BmsTelemetry::Faults { source, code, .. } => {
                self.push_feed(source, "BMS_FAULTS", format!("code {code}"));
                self.ensure_device(source);
            }
        }
    }

    fn push_status(&mut self, status: String) {
        self.status = status.clone();
        self.push_feed(Device::Raspi, "TUI_STATUS", status);
    }

    fn push_feed(&mut self, device: Device, command: &'static str, detail: String) {
        self.ensure_device(device);
        self.feed.push_back(FeedLine {
            time: Local::now().format("%H:%M:%S").to_string(),
            device,
            command,
            detail,
        });

        while self.feed.len() > MAX_FEED_LINES {
            self.feed.pop_front();
        }
    }

    fn ensure_device(&mut self, device: Device) {
        if !self.devices.contains(&device) {
            self.devices.push(device);
            self.devices.sort_by_key(|device| device.label());
        }

        if self.selected_device >= self.devices.len() {
            self.selected_device = self.devices.len().saturating_sub(1);
        }
    }

    fn select_first_telemetry_source(&mut self, device: Device) {
        if !self.series.is_empty() {
            return;
        }

        self.ensure_device(device);
        if let Some(index) = self
            .devices
            .iter()
            .position(|candidate| *candidate == device)
        {
            self.selected_device = index;
        }
    }

    fn next_x(&mut self) -> f64 {
        self.sample_index += 1.0;
        self.sample_index
    }

    fn push_point(&mut self, device: Device, sensor: Sensor, x: f64, y: f64) {
        if !y.is_finite() {
            return;
        }

        let values = self.series.entry(SeriesKey { device, sensor }).or_default();
        values.push_back((x, y));

        while values.len() > MAX_SERIES_POINTS {
            values.pop_front();
        }
    }

    fn selected_device(&self) -> Option<Device> {
        self.devices.get(self.selected_device).copied()
    }

    fn selected_sensor(&self) -> Sensor {
        Sensor::ALL[self.selected_sensor]
    }

    fn selected_points(&self) -> Vec<(f64, f64)> {
        let Some(device) = self.selected_device() else {
            return Vec::new();
        };

        self.series
            .get(&SeriesKey {
                device,
                sensor: self.selected_sensor(),
            })
            .map(|values| values.iter().copied().collect())
            .unwrap_or_default()
    }

    fn previous_device(&mut self) {
        if self.devices.is_empty() {
            return;
        }

        self.selected_device = if self.selected_device == 0 {
            self.devices.len() - 1
        } else {
            self.selected_device - 1
        };
    }

    fn next_device(&mut self) {
        if !self.devices.is_empty() {
            self.selected_device = (self.selected_device + 1) % self.devices.len();
        }
    }

    fn previous_sensor(&mut self) {
        self.selected_sensor = if self.selected_sensor == 0 {
            Sensor::ALL.len() - 1
        } else {
            self.selected_sensor - 1
        };
    }

    fn next_sensor(&mut self) {
        self.selected_sensor = (self.selected_sensor + 1) % Sensor::ALL.len();
    }

    fn clear_selected_series(&mut self) {
        let Some(device) = self.selected_device() else {
            return;
        };

        self.series.remove(&SeriesKey {
            device,
            sensor: self.selected_sensor(),
        });
    }

    fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

fn average(values: impl Iterator<Item = f64>) -> f64 {
    let mut sum = 0.0;
    let mut count = 0.0;

    for value in values {
        sum += value;
        count += 1.0;
    }

    if count == 0.0 { 0.0 } else { sum / count }
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    draw_header(frame, root[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(root[1]);

    draw_feed(frame, body[0], app);
    draw_graph(frame, body[1], app);
    draw_footer(frame, root[2], app);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let state = if app.connected { "LIVE" } else { "OFFLINE" };
    let style = if app.connected {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "CAN Websocket Visualizer  ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(state, style),
        Span::raw(format!("  {}  uptime {}s", app.url, app.uptime_seconds())),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_feed(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let height = area.height.saturating_sub(2) as usize;
    let items = app.feed.iter().rev().take(height).rev().map(|line| {
        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", line.time),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{:<14}", line.device.label()),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("{:<18}", line.command),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw(line.detail.clone()),
        ]))
    });

    let list = List::new(items.collect::<Vec<_>>()).block(
        Block::default()
            .title(" Parsed CAN Stream ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, area);
}

fn draw_graph(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let points = app.selected_points();
    let device = app
        .selected_device()
        .map(|device| device.label())
        .unwrap_or("no nodes yet");
    let sensor = app.selected_sensor();
    let title = format!(" Graph  {device} / {} ", sensor.label());
    let datasets = vec![
        Dataset::default()
            .name(sensor.label())
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&points),
    ];

    let (x_bounds, y_bounds) = chart_bounds(&points);
    let chart = Chart::new(datasets)
        .block(Block::default().title(title).borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("sample")
                .style(Style::default().fg(Color::Gray))
                .bounds(x_bounds),
        )
        .y_axis(
            Axis::default()
                .title(sensor.unit())
                .style(Style::default().fg(Color::Gray))
                .bounds(y_bounds),
        );

    frame.render_widget(chart, area);
}

fn chart_bounds(points: &[(f64, f64)]) -> ([f64; 2], [f64; 2]) {
    if points.is_empty() {
        return ([0.0, MAX_SERIES_POINTS as f64], [-1.0, 1.0]);
    }

    let x_min = points.first().map(|point| point.0).unwrap_or(0.0);
    let x_max = points.last().map(|point| point.0).unwrap_or(1.0);
    let (mut y_min, mut y_max) = points
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), point| {
            (min.min(point.1), max.max(point.1))
        });

    if (y_max - y_min).abs() < f64::EPSILON {
        y_min -= 1.0;
        y_max += 1.0;
    } else {
        let padding = (y_max - y_min) * 0.15;
        y_min -= padding;
        y_max += padding;
    }

    ([x_min, x_max.max(x_min + 1.0)], [y_min, y_max])
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let footer = Paragraph::new(Line::from(vec![
        Span::raw("node "),
        Span::styled("left/right", Style::default().fg(Color::Cyan)),
        Span::raw("  sensor "),
        Span::styled("up/down", Style::default().fg(Color::Cyan)),
        Span::raw("  clear "),
        Span::styled("c", Style::default().fg(Color::Cyan)),
        Span::raw("  quit "),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::raw(format!("  |  {}", app.status)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackendEvent {
    data: BackendEventData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum BackendEventData {
    DeviceRegistrySnapshot { devices: Vec<DeviceStatus> },
    DeviceStatusChanged { device: DeviceStatus },
    DaqTelemetry { telemetry: DaqTelemetry },
    BmsTelemetry { telemetry: BmsTelemetry },
    BackendError { message: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum DaqTelemetry {
    Temperature {
        source: Device,
        samples: Vec<TemperatureSample>,
    },
    Imu {
        source: Device,
        samples: Vec<ImuSample>,
    },
    #[serde(rename = "tbd")]
    Tbd { source: Device, value: f64 },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum BmsTelemetry {
    Voltages {
        source: Device,
        readings: BmsVoltageReadings,
    },
    Temperatures {
        source: Device,
        readings: BmsTemperatureReadings,
    },
    Balancing {
        source: Device,
        #[serde(rename = "activeCell")]
        _active_cell: u8,
        #[serde(rename = "dutyCycle")]
        _duty_cycle: f64,
    },
    Faults {
        source: Device,
        code: u32,
        _severity: f64,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemperatureSample {
    tire: f64,
    brake: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImuSample {
    acceleration: Acceleration,
    angular_acceleration: AngularAcceleration,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Acceleration {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AngularAcceleration {
    rho: f64,
    theta: f64,
    phi: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BmsVoltageReadings {
    pack: f64,
    _min_cell: f64,
    _max_cell: f64,
    _average_cell: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BmsTemperatureReadings {
    _min: f64,
    _max: f64,
    average: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceStatus {
    node_id: u8,
    name: String,
    online: bool,
}

impl DeviceStatus {
    fn device(&self) -> Device {
        Device::from_node_id(self.node_id).unwrap_or(Device::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Device {
    Bms,
    Vcu,
    Raspi,
    NodeFL,
    NodeFR,
    NodeRL,
    NodeRR,
    NodeDash,
    Nucleo1,
    Nucleo2,
    NodePDMDASH,
    NodePDMPCBPanel,
    #[serde(other)]
    Unknown,
}

impl Device {
    fn from_node_id(node_id: u8) -> Option<Self> {
        match node_id {
            0x02 => Some(Self::NodeFL),
            0x03 => Some(Self::NodeFR),
            0x04 => Some(Self::NodeRL),
            0x05 => Some(Self::NodeRR),
            0x06 => Some(Self::Nucleo1),
            0x07 => Some(Self::Nucleo2),
            0x1D => Some(Self::NodeDash),
            0x1E => Some(Self::Raspi),
            0x1F => Some(Self::Bms),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Bms => "BMS",
            Self::Vcu => "VCU",
            Self::Raspi => "Raspi",
            Self::NodeFL => "Front Left",
            Self::NodeFR => "Front Right",
            Self::NodeRL => "Rear Left",
            Self::NodeRR => "Rear Right",
            Self::NodeDash => "Dash",
            Self::Nucleo1 => "DAQ Nucleo 1",
            Self::Nucleo2 => "DAQ Nucleo 2",
            Self::NodePDMDASH => "PDM Dash",
            Self::NodePDMPCBPanel => "PDM Panel",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
struct FeedLine {
    time: String,
    device: Device,
    command: &'static str,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SeriesKey {
    device: Device,
    sensor: Sensor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Sensor {
    AccelX,
    AccelY,
    AccelZ,
    GyroRho,
    GyroTheta,
    GyroPhi,
    TireAverage,
    BrakeAverage,
    BmsPackVoltage,
    BmsAverageTemp,
}

impl Sensor {
    const ALL: [Self; 10] = [
        Self::AccelX,
        Self::AccelY,
        Self::AccelZ,
        Self::GyroRho,
        Self::GyroTheta,
        Self::GyroPhi,
        Self::TireAverage,
        Self::BrakeAverage,
        Self::BmsPackVoltage,
        Self::BmsAverageTemp,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::AccelX => "IMU accel X",
            Self::AccelY => "IMU accel Y",
            Self::AccelZ => "IMU accel Z",
            Self::GyroRho => "IMU gyro rho",
            Self::GyroTheta => "IMU gyro theta",
            Self::GyroPhi => "IMU gyro phi",
            Self::TireAverage => "tire temp avg",
            Self::BrakeAverage => "brake temp avg",
            Self::BmsPackVoltage => "BMS pack voltage",
            Self::BmsAverageTemp => "BMS temp avg",
        }
    }

    fn unit(self) -> &'static str {
        match self {
            Self::AccelX | Self::AccelY | Self::AccelZ => "g",
            Self::GyroRho | Self::GyroTheta | Self::GyroPhi => "dps",
            Self::TireAverage | Self::BrakeAverage | Self::BmsAverageTemp => "degC",
            Self::BmsPackVoltage => "V",
        }
    }
}
