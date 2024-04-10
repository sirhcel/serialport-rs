use clap::Parser;
use std::io::Write;

#[derive(Debug, Parser)]
struct Config {
    device: String,
    baud: u32,
}

fn main() {
    let config = Config::parse();
    let mut port = serialport::new(config.device, config.baud)
        .baud_rate(config.baud)
        .open()
        .unwrap();

    let data = b"Hello world!\r\n";

    port.write_all(data).unwrap();
    port.flush().unwrap();
}
