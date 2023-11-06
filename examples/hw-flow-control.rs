use clap::Parser;
use log::info;
use serialport::FlowControl;
use serialport::SerialPort;
use serialport::TTYPort;

#[derive(Debug, Parser)]
struct Config {
    #[clap(long, default_value_t = 115200)]
    baud: u32,
    path: String,
}

pub fn main() {
    env_logger::init();

    let config = Config::parse();

    let mut serial_port = TTYPort::open(
        &serialport::new(config.path, config.baud).flow_control(FlowControl::Hardware),
    )
    .expect("Could not open serial port.");

    info!(
        "Serial port path: {}",
        serial_port.name().expect("Could not get port name.")
    );
    info!(
        "Serial port baud rate: {}",
        serial_port.baud_rate().expect("Could not get baud rate.")
    );
    info!(
        "Serial port flow control: {}",
        serial_port
            .flow_control()
            .expect("Could not get flow control.")
    );
    serial_port
        .set_flow_control(FlowControl::Hardware)
        .expect("Could not set flow control.");
    info!(
        "Serial port flow control: {}",
        serial_port
            .flow_control()
            .expect("Could not get flow control.")
    );
    serial_port
        .set_flow_control(FlowControl::Software)
        .expect("Could not set flow control.");
    info!(
        "Serial port flow control: {}",
        serial_port
            .flow_control()
            .expect("Could not get flow control.")
    );
    serial_port
        .set_flow_control(FlowControl::Hardware)
        .expect("Could not set flow control.");
    info!(
        "Serial port flow control: {}",
        serial_port
            .flow_control()
            .expect("Could not get flow control.")
    );
    serial_port
        .set_flow_control(FlowControl::None)
        .expect("Could not set flow control.");
    info!(
        "Serial port flow control: {}",
        serial_port
            .flow_control()
            .expect("Could not get flow control.")
    );
}
