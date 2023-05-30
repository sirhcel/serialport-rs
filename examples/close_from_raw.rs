//! This example shows how to close a serial port passed as raw pointer (for example through FFI).

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value_t = 9600)]
    baud: u32,
    #[clap(short, long)]
    port: String,
}

fn main() {
    let args = Args::parse();

    let port = serialport::new(args.port, args.baud).open().unwrap();
    println!("Opened serial port {:?}", port);
    let raw = Box::into_raw(port);
    println!("Converted port into raw pointer {:?}", raw);
    let port = unsafe { Box::from_raw(raw) };
    println!("Converted raw pointer back to port {:?}", port);
    drop(port);
    println!("Dropped (and closed) port.");
}
