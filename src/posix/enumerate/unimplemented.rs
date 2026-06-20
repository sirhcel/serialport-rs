/// Enumerating serial ports on this platform is not supported
pub fn available_ports() -> Result<Vec<SerialPortInfo>> {
    Err(Error::new(
        ErrorKind::Unknown,
        "Not implemented for this OS",
    ))
}
