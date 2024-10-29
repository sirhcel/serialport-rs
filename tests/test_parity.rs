mod config;

use config::{hw_config, HardwareConfig};
use rstest::rstest;
use rstest_reuse::{self, apply, template};
use serialport::{ClearBuffer, DataBits, Parity};

// TODO: Add tests for different ways of setting parity.

// TODO: Add test for parity errors.

#[template]
#[rstest]
#[case(Parity::None)]
#[case(Parity::Odd)]
#[case(Parity::Even)]
fn standard_parities(#[case] parity: Parity) {}

#[apply(standard_parities)]
fn test_end_to_end(hw_config: HardwareConfig, #[case] parity: Parity) {
    const MESSAGE: [u8; 10] = *b"0123456789";

    let mut sender = serialport::new(hw_config.port_1, 115200)
        .parity(parity)
        .open()
        .unwrap();
    let mut receiver = serialport::new(hw_config.port_2, 115200)
        .parity(parity)
        .open()
        .unwrap();

    sender.clear(ClearBuffer::All).unwrap();
    receiver.clear(ClearBuffer::All).unwrap();

    sender.write_all(&MESSAGE).unwrap();
    sender.flush().unwrap();

    let mut buffer = [0u8; MESSAGE.len()];
    receiver.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, MESSAGE);
}

#[rstest]
#[case(Parity::Odd, b"\xb0\x31\x32\xb3\x34\xb5\xb6\x37\x38\xb9")]
#[case(Parity::Even, b"\x30\xb1\xb2\x33\xb4\x35\x36\xb7\xb8\x39")]
fn test_emulated_standard_parity(
    hw_config: HardwareConfig,
    #[case] parity: Parity,
    #[case] message_with_parity: &[u8],
) {
    const MESSAGE: [u8; 10] = *b"0123456789";

    // Emulate parity bit through the eight data bit.
    let mut sender = serialport::new(hw_config.port_1, 115200)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .open()
        .unwrap();
    // Let the OS handle parity checking and removal by configuring only seven data bits.
    let mut receiver = serialport::new(hw_config.port_2, 115200)
        .data_bits(DataBits::Seven)
        .parity(parity)
        .open()
        .unwrap();

    sender.clear(ClearBuffer::All).unwrap();
    receiver.clear(ClearBuffer::All).unwrap();

    sender.write_all(message_with_parity).unwrap();
    sender.flush().unwrap();

    let mut buffer = [0u8; MESSAGE.len()];
    receiver.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, MESSAGE);
}

#[rstest]
#[case(Parity::Mark, b"\xb0\xb1\xb2\xb3\xb4\xb5\xb6\xb7\xb8\xb9")]
#[case(Parity::Space, b"0123456789")]
fn test_emulated_mark_space_parity(
    hw_config: HardwareConfig,
    #[case] parity: Parity,
    #[case] message_with_parity: &[u8],
) {
    const MESSAGE: [u8; 10] = *b"0123456789";

    // Emulate parity bit through the eight data bit.
    let mut sender = serialport::new(hw_config.port_1, 115200)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .open()
        .unwrap();
    // Let the OS handle parity checking and removal by configuring only seven data bits.
    let mut receiver = serialport::new(hw_config.port_2, 115200)
        .data_bits(DataBits::Seven)
        .parity(parity)
        .open()
        .unwrap();

    sender.clear(ClearBuffer::All).unwrap();
    receiver.clear(ClearBuffer::All).unwrap();

    sender.write_all(message_with_parity).unwrap();
    sender.flush().unwrap();

    let mut buffer = [0u8; MESSAGE.len()];
    receiver.read_exact(&mut buffer).unwrap();

    assert_eq!(buffer, MESSAGE);
}
