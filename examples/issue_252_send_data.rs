use std::{thread::sleep, time::Duration};

fn main() {
    let port_name = "COM6"; // Adjust for your setup
    let baud_rate = 115200;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_secs(1))
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .flow_control(serialport::FlowControl::None)
        .open()
        .expect("Failed to open serial port");

    let tosend: Vec<u8> = Vec::from_iter(50..=255);

    for b in tosend {
        port.write(&[b]).expect("Failed to write byte");
        port.flush().unwrap();

        sleep(Duration::from_millis(200));
    }
}
