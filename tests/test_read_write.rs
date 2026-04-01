mod config;

use config::{hw_config, HardwareConfig};
use rstest::rstest;

const DEFAULT_BAUD: u32 = 115_200;

mod physical {
    use super::*;

    #[rstest]
    fn test_read_zero_bytes(hw_config: HardwareConfig) {
        let mut port = serialport::new(hw_config.port_1, DEFAULT_BAUD)
            .open()
            .unwrap();
        let mut data = [];

        let read = port.read(&mut data).unwrap();
        assert_eq!(read, 0);
    }

    #[rstest]
    fn test_write_zero_bytes(hw_config: HardwareConfig) {
        let mut port = serialport::new(hw_config.port_1, DEFAULT_BAUD)
            .open()
            .unwrap();

        let written = port.write(&[]).unwrap();
        assert_eq!(written, 0);
        port.flush().unwrap();
    }
}

#[cfg(unix)]
mod pseudo {
    use super::*;

    use serialport::TTYPort;
    use std::io::{Read, Write};

    #[rstest]
    fn test_read_zero_bytes() {
        let (mut master, _slave) = TTYPort::pair().unwrap();
        let mut data = [];

        let read = master.read(&mut data).unwrap();
        assert_eq!(read, 0);
    }

    #[rstest]
    fn test_write_zero_bytes() {
        let (mut master, _slave) = TTYPort::pair().unwrap();

        let written = master.write(&[]).unwrap();
        assert_eq!(written, 0);
        master.flush().unwrap();
    }
}
