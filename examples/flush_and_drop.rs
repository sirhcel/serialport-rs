use clap::Parser;
use serialport::ClearBuffer;
use std::io::Write;
use std::time::Duration;

/// Test for getting all data written to the wire when shutting down the application.
#[derive(Clone, Debug, Parser)]
struct Args {
    port: String,
}

fn main() {
    let args = Args::parse();

    let mut port = serialport::new(args.port, 9600).open().unwrap();
    port.clear(ClearBuffer::All).unwrap();

    let first: &[u8] = &[
        0x01, 0x00, 0x05, 0x34, 0x00, 0x00, 0x25, 0x80, 0x22, 0x03, 0xfe,
    ];
    let second: &[u8] = &[0x01, 0x00, 0x01, 0x00, 0xff, 0x03];

    port.write_all(first).unwrap();
    port.flush().unwrap();
    std::thread::sleep(Duration::from_millis(4));
    port.write_all(second).unwrap();
}
