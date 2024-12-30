use clap::Parser;
use serialport::{DataBits, FlowControl, StopBits};
use std::time::Duration;

#[derive(Debug, Parser)]
pub struct Cli {
    device: String,
}

fn main() {
    let cli = Cli::parse();
    let mut port = serialport::new(cli.device, 115_200)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_millis(3000))
        .open()
        .unwrap();

    port.write_data_terminal_ready(true).unwrap();
    port.write_request_to_send(true).unwrap();

    port.write_all(
        b"0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    )
    .unwrap();
    port.flush().unwrap();
}
