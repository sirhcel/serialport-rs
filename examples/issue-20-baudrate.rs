use clap::Parser;
use std::time::Duration;

#[derive(Debug, Parser)]
struct Args {
    baud: u32,
    port: String,
}

pub fn main() {
    let args = Args::parse();

    let port = serialport::new(args.port, args.baud);
    let port = port
        .timeout(Duration::from_secs(5))
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None);
    let mut port = port.open().unwrap();

    let message =
        "0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"
            .as_bytes();
    port.write_all(message).unwrap();
    port.flush().unwrap();

    #[cfg(target_os = "macos")]
    std::thread::sleep(Duration::from_secs(1));
}
