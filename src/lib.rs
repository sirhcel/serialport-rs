//! serialport-rs is a cross-platform serial port library.
//!
//! The goal of this library is to expose a cross-platform and platform-specific API for enumerating
//! and using blocking I/O with serial ports. This library exposes a similar API to that provided
//! by [Qt's `QSerialPort` library](https://doc.qt.io/qt-5/qserialport.html).
//!
//! # Feature Overview
//!
//! The library has been organized such that there is a high-level `SerialPort` trait that provides
//! a cross-platform API for accessing serial ports. This is the preferred method of interacting
//! with ports. The `SerialPort::new().open*()` and `available_ports()` functions in the root
//! provide cross-platform functionality.
//!
//! For platform-specific functionality, this crate is split into a `posix` and `windows` API with
//! corresponding `TTYPort` and `COMPort` structs (that both implement the `SerialPort` trait).
//! Using the platform-specific `SerialPort::new().open*()` functions will return the
//! platform-specific port object which allows access to platform-specific functionality.

#![deny(
    clippy::dbg_macro,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
// Document feature-gated elements on docs.rs. See
// https://doc.rust-lang.org/rustdoc/unstable-features.html?highlight=doc(cfg#doccfg-recording-what-platforms-or-features-are-required-for-code-to-be-present
// and
// https://doc.rust-lang.org/rustdoc/unstable-features.html#doc_auto_cfg-automatically-generate-doccfg
// for details.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// Don't worry about needing to `unwrap()` or otherwise handle some results in
// doc tests.
#![doc(test(attr(allow(unused_must_use))))]

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::str::FromStr;
use std::time::Duration;

#[cfg(unix)]
mod posix;
#[cfg(unix)]
pub use posix::{BreakDuration, TTYPort};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::COMPort;

#[cfg(test)]
pub(crate) mod tests;

/// A type for results generated by interacting with serial ports
///
/// The `Err` type is hard-wired to [`serialport::Error`](struct.Error.html).
pub type Result<T> = std::result::Result<T, Error>;

/// Categories of errors that can occur when interacting with serial ports
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The device is not available.
    ///
    /// This could indicate that the device is in use by another process or was
    /// disconnected while performing I/O.
    NoDevice,

    /// A parameter was incorrect.
    InvalidInput,

    /// An unknown error occurred.
    Unknown,

    /// An I/O error occurred.
    ///
    /// The type of I/O error is determined by the inner `io::ErrorKind`.
    Io(io::ErrorKind),
}

/// An error type for serial port operations
#[derive(Debug, Clone)]
pub struct Error {
    /// The kind of error this is
    pub kind: ErrorKind,
    /// A description of the error suitable for end-users
    pub description: String,
}

impl Error {
    /// Instantiates a new error
    pub fn new<T: Into<String>>(kind: ErrorKind, description: T) -> Self {
        Error {
            kind,
            description: description.into(),
        }
    }

    /// Returns the corresponding `ErrorKind` for this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_str(&self.description)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::new(ErrorKind::Io(io_error.kind()), format!("{}", io_error))
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        let kind = match error.kind {
            ErrorKind::NoDevice => io::ErrorKind::NotFound,
            ErrorKind::InvalidInput => io::ErrorKind::InvalidInput,
            ErrorKind::Unknown => io::ErrorKind::Other,
            ErrorKind::Io(kind) => kind,
        };

        io::Error::new(kind, error.description)
    }
}

/// Number of bits per character
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DataBits {
    /// 5 bits per character
    Five,

    /// 6 bits per character
    Six,

    /// 7 bits per character
    Seven,

    /// 8 bits per character
    Eight,
}

impl fmt::Display for DataBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DataBits::Five => write!(f, "Five"),
            DataBits::Six => write!(f, "Six"),
            DataBits::Seven => write!(f, "Seven"),
            DataBits::Eight => write!(f, "Eight"),
        }
    }
}

impl From<DataBits> for u8 {
    fn from(value: DataBits) -> Self {
        match value {
            DataBits::Five => 5,
            DataBits::Six => 6,
            DataBits::Seven => 7,
            DataBits::Eight => 8,
        }
    }
}

impl TryFrom<u8> for DataBits {
    type Error = ();

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            _ => Err(()),
        }
    }
}

/// Parity checking modes
///
/// When parity checking is enabled (`Odd` or `Even`) an extra bit is transmitted with
/// each character. The value of the parity bit is arranged so that the number of 1 bits in the
/// character (including the parity bit) is an even number (`Even`) or an odd number
/// (`Odd`).
///
/// Parity checking is disabled by setting `None`, in which case parity bits are not
/// transmitted.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parity {
    /// No parity bit.
    None,

    /// Parity bit sets odd number of 1 bits.
    Odd,

    /// Parity bit sets even number of 1 bits.
    Even,
}

impl fmt::Display for Parity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Parity::None => write!(f, "None"),
            Parity::Odd => write!(f, "Odd"),
            Parity::Even => write!(f, "Even"),
        }
    }
}

/// Number of stop bits
///
/// Stop bits are transmitted after every character.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StopBits {
    /// One stop bit.
    One,

    /// Two stop bits.
    Two,
}

impl fmt::Display for StopBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StopBits::One => write!(f, "One"),
            StopBits::Two => write!(f, "Two"),
        }
    }
}

impl From<StopBits> for u8 {
    fn from(value: StopBits) -> Self {
        match value {
            StopBits::One => 1,
            StopBits::Two => 2,
        }
    }
}

impl TryFrom<u8> for StopBits {
    type Error = ();

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(()),
        }
    }
}

/// Flow control modes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlowControl {
    /// No flow control.
    None,

    /// Flow control using XON/XOFF bytes.
    Software,

    /// Flow control using RTS/CTS signals.
    Hardware,
}

impl fmt::Display for FlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            FlowControl::None => write!(f, "None"),
            FlowControl::Software => write!(f, "Software"),
            FlowControl::Hardware => write!(f, "Hardware"),
        }
    }
}

impl FromStr for FlowControl {
    type Err = ();

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s {
            "None" | "none" | "n" => Ok(FlowControl::None),
            "Software" | "software" | "SW" | "sw" | "s" => Ok(FlowControl::Software),
            "Hardware" | "hardware" | "HW" | "hw" | "h" => Ok(FlowControl::Hardware),
            _ => Err(()),
        }
    }
}

/// Specifies which buffer or buffers to purge when calling [`clear`]
///
/// [`clear`]: trait.SerialPort.html#tymethod.clear
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClearBuffer {
    /// Specify to clear data received but not read
    Input,
    /// Specify to clear data written but not yet transmitted
    Output,
    /// Specify to clear both data received and data not yet transmitted
    All,
}

/// A struct containing all serial port settings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialPortBuilder {
    /// The port name, usually the device path
    path: String,
    /// The baud rate in symbols-per-second
    baud_rate: u32,
    /// Number of bits used to represent a character sent on the line
    data_bits: DataBits,
    /// The type of signalling to use for controlling data transfer
    flow_control: FlowControl,
    /// The type of parity to use for error checking
    parity: Parity,
    /// Number of bits to use to signal the end of a character
    stop_bits: StopBits,
    /// Amount of time to wait to receive data before timing out
    timeout: Duration,
}

impl SerialPortBuilder {
    /// Set the path to the serial port
    // TODO: Switch to `clone_into` when bumping our MSRV past 1.63 and remove this exemption.
    #[allow(clippy::assigning_clones)]
    #[must_use]
    pub fn path<'a>(mut self, path: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.path = path.into().as_ref().to_owned();
        self
    }

    /// Set the baud rate in symbols-per-second
    #[must_use]
    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// Set the number of bits used to represent a character sent on the line
    #[must_use]
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the type of signalling to use for controlling data transfer
    #[must_use]
    pub fn flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Set the type of parity to use for error checking
    #[must_use]
    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the number of bits to use to signal the end of a character
    #[must_use]
    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Set the amount of time to wait to receive data before timing out
    ///
    /// <div class="warning">
    ///
    /// The accuracy is limited by the underlying platform's capabilities. Longer timeouts will be
    /// clamped to the maximum supported value which is expected to be in the magnitude of a few
    /// days.
    ///
    /// </div>
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Open a cross-platform interface to the port with the specified settings
    pub fn open(self) -> Result<Box<dyn SerialPort>> {
        #[cfg(unix)]
        return posix::TTYPort::open(&self).map(|p| Box::new(p) as Box<dyn SerialPort>);

        #[cfg(windows)]
        return windows::COMPort::open(&self).map(|p| Box::new(p) as Box<dyn SerialPort>);

        #[cfg(not(any(unix, windows)))]
        Err(Error::new(
            ErrorKind::Unknown,
            "open() not implemented for platform",
        ))
    }

    /// Open a platform-specific interface to the port with the specified settings
    #[cfg(unix)]
    pub fn open_native(self) -> Result<TTYPort> {
        posix::TTYPort::open(&self)
    }

    /// Open a platform-specific interface to the port with the specified settings
    #[cfg(windows)]
    pub fn open_native(self) -> Result<COMPort> {
        windows::COMPort::open(&self)
    }
}

/// A trait for serial port devices
///
/// This trait is all that's necessary to implement a new serial port driver
/// for a new platform.
pub trait SerialPort: Send + Sync + io::Read + io::Write {
    // Port settings getters

    /// Returns the name of this port if it exists.
    ///
    /// This name may not be the canonical device name and instead be shorthand.
    /// Additionally it may not exist for virtual ports.
    fn name(&self) -> Option<String>;

    /// Returns the current baud rate.
    ///
    /// This may return a value different from the last specified baud rate depending on the
    /// platform as some will return the actual device baud rate rather than the last specified
    /// baud rate.
    fn baud_rate(&self) -> Result<u32>;

    /// Returns the character size.
    ///
    /// This function returns `None` if the character size could not be determined. This may occur
    /// if the hardware is in an uninitialized state or is using a non-standard character size.
    /// Setting a baud rate with `set_char_size()` should initialize the character size to a
    /// supported value.
    fn data_bits(&self) -> Result<DataBits>;

    /// Returns the flow control mode.
    ///
    /// This function returns `None` if the flow control mode could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported flow control
    /// mode. Setting a flow control mode with `set_flow_control()` should initialize the flow
    /// control mode to a supported value.
    fn flow_control(&self) -> Result<FlowControl>;

    /// Returns the parity-checking mode.
    ///
    /// This function returns `None` if the parity mode could not be determined. This may occur if
    /// the hardware is in an uninitialized state or is using a non-standard parity mode. Setting
    /// a parity mode with `set_parity()` should initialize the parity mode to a supported value.
    fn parity(&self) -> Result<Parity>;

    /// Returns the number of stop bits.
    ///
    /// This function returns `None` if the number of stop bits could not be determined. This may
    /// occur if the hardware is in an uninitialized state or is using an unsupported stop bit
    /// configuration. Setting the number of stop bits with `set_stop-bits()` should initialize the
    /// stop bits to a supported value.
    fn stop_bits(&self) -> Result<StopBits>;

    /// Returns the current timeout.
    fn timeout(&self) -> Duration;

    // Port settings setters

    /// Sets the baud rate.
    ///
    /// ## Errors
    ///
    /// If the implementation does not support the requested baud rate, this function may return an
    /// `InvalidInput` error. Even if the baud rate is accepted by `set_baud_rate()`, it may not be
    /// supported by the underlying hardware.
    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()>;

    /// Sets the character size.
    fn set_data_bits(&mut self, data_bits: DataBits) -> Result<()>;

    /// Sets the flow control mode.
    fn set_flow_control(&mut self, flow_control: FlowControl) -> Result<()>;

    /// Sets the parity-checking mode.
    fn set_parity(&mut self, parity: Parity) -> Result<()>;

    /// Sets the number of stop bits.
    fn set_stop_bits(&mut self, stop_bits: StopBits) -> Result<()>;

    /// Sets the timeout for future I/O operations.
    ///
    /// <div class="warning">
    ///
    /// The accuracy is limited by the underlying platform's capabilities. Longer timeouts will be
    /// clamped to the maximum supported value which is expected to be in the magnitude of a few
    /// days.
    ///
    /// </div>
    fn set_timeout(&mut self, timeout: Duration) -> Result<()>;

    // Functions for setting non-data control signal pins

    /// Sets the state of the RTS (Request To Send) control signal.
    ///
    /// Setting a value of `true` asserts the RTS control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the RTS control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn write_request_to_send(&mut self, level: bool) -> Result<()>;

    /// Writes to the Data Terminal Ready pin
    ///
    /// Setting a value of `true` asserts the DTR control signal. `false` clears the signal.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the DTR control signal could not be set to the desired
    /// state on the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn write_data_terminal_ready(&mut self, level: bool) -> Result<()>;

    // Functions for reading additional pins

    /// Reads the state of the CTS (Clear To Send) control signal.
    ///
    /// This function returns a boolean that indicates whether the CTS control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CTS control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_clear_to_send(&mut self) -> Result<bool>;

    /// Reads the state of the Data Set Ready control signal.
    ///
    /// This function returns a boolean that indicates whether the DSR control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the DSR control signal could not be read
    /// from the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_data_set_ready(&mut self) -> Result<bool>;

    /// Reads the state of the Ring Indicator control signal.
    ///
    /// This function returns a boolean that indicates whether the RI control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the RI control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_ring_indicator(&mut self) -> Result<bool>;

    /// Reads the state of the Carrier Detect control signal.
    ///
    /// This function returns a boolean that indicates whether the CD control signal is asserted.
    ///
    /// ## Errors
    ///
    /// This function returns an error if the state of the CD control signal could not be read from
    /// the underlying hardware:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn read_carrier_detect(&mut self) -> Result<bool>;

    /// Gets the number of bytes available to be read from the input buffer.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn bytes_to_read(&self) -> Result<u32>;

    /// Get the number of bytes written to the output buffer, awaiting transmission.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn bytes_to_write(&self) -> Result<u32>;

    /// Discards all bytes from the serial driver's input buffer and/or output buffer.
    ///
    /// # Errors
    ///
    /// This function may return the following errors:
    ///
    /// * `NoDevice` if the device was disconnected.
    /// * `Io` for any other type of I/O error.
    fn clear(&self, buffer_to_clear: ClearBuffer) -> Result<()>;

    // Misc methods

    /// Attempts to clone the `SerialPort`. This allow you to write and read simultaneously from the
    /// same serial connection. Please note that if you want a real asynchronous serial port you
    /// should look at [mio-serial](https://crates.io/crates/mio-serial) or
    /// [tokio-serial](https://crates.io/crates/tokio-serial).
    ///
    /// Also, you must be very careful when changing the settings of a cloned `SerialPort` : since
    /// the settings are cached on a per object basis, trying to modify them from two different
    /// objects can cause some nasty behavior.
    ///
    /// # Errors
    ///
    /// This function returns an error if the serial port couldn't be cloned.
    fn try_clone(&self) -> Result<Box<dyn SerialPort>>;

    /// Start transmitting a break
    fn set_break(&self) -> Result<()>;

    /// Stop transmitting a break
    fn clear_break(&self) -> Result<()>;
}

impl<T: SerialPort> SerialPort for &mut T {
    fn name(&self) -> Option<String> {
        (**self).name()
    }

    fn baud_rate(&self) -> Result<u32> {
        (**self).baud_rate()
    }

    fn data_bits(&self) -> Result<DataBits> {
        (**self).data_bits()
    }

    fn flow_control(&self) -> Result<FlowControl> {
        (**self).flow_control()
    }

    fn parity(&self) -> Result<Parity> {
        (**self).parity()
    }

    fn stop_bits(&self) -> Result<StopBits> {
        (**self).stop_bits()
    }

    fn timeout(&self) -> Duration {
        (**self).timeout()
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> Result<()> {
        (**self).set_baud_rate(baud_rate)
    }

    fn set_data_bits(&mut self, data_bits: DataBits) -> Result<()> {
        (**self).set_data_bits(data_bits)
    }

    fn set_flow_control(&mut self, flow_control: FlowControl) -> Result<()> {
        (**self).set_flow_control(flow_control)
    }

    fn set_parity(&mut self, parity: Parity) -> Result<()> {
        (**self).set_parity(parity)
    }

    fn set_stop_bits(&mut self, stop_bits: StopBits) -> Result<()> {
        (**self).set_stop_bits(stop_bits)
    }

    fn set_timeout(&mut self, timeout: Duration) -> Result<()> {
        (**self).set_timeout(timeout)
    }

    fn write_request_to_send(&mut self, level: bool) -> Result<()> {
        (**self).write_request_to_send(level)
    }

    fn write_data_terminal_ready(&mut self, level: bool) -> Result<()> {
        (**self).write_data_terminal_ready(level)
    }

    fn read_clear_to_send(&mut self) -> Result<bool> {
        (**self).read_clear_to_send()
    }

    fn read_data_set_ready(&mut self) -> Result<bool> {
        (**self).read_data_set_ready()
    }

    fn read_ring_indicator(&mut self) -> Result<bool> {
        (**self).read_ring_indicator()
    }

    fn read_carrier_detect(&mut self) -> Result<bool> {
        (**self).read_carrier_detect()
    }

    fn bytes_to_read(&self) -> Result<u32> {
        (**self).bytes_to_read()
    }

    fn bytes_to_write(&self) -> Result<u32> {
        (**self).bytes_to_write()
    }

    fn clear(&self, buffer_to_clear: ClearBuffer) -> Result<()> {
        (**self).clear(buffer_to_clear)
    }

    fn try_clone(&self) -> Result<Box<dyn SerialPort>> {
        (**self).try_clone()
    }

    fn set_break(&self) -> Result<()> {
        (**self).set_break()
    }

    fn clear_break(&self) -> Result<()> {
        (**self).clear_break()
    }
}

impl fmt::Debug for dyn SerialPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SerialPort ( ")?;

        if let Some(n) = self.name().as_ref() {
            write!(f, "name: {} ", n)?;
        };
        if let Ok(b) = self.baud_rate().as_ref() {
            write!(f, "baud_rate: {} ", b)?;
        };
        if let Ok(b) = self.data_bits().as_ref() {
            write!(f, "data_bits: {} ", b)?;
        };
        if let Ok(c) = self.flow_control().as_ref() {
            write!(f, "flow_control: {} ", c)?;
        }
        if let Ok(p) = self.parity().as_ref() {
            write!(f, "parity: {} ", p)?;
        }
        if let Ok(s) = self.stop_bits().as_ref() {
            write!(f, "stop_bits: {} ", s)?;
        }

        write!(f, ")")
    }
}

/// Contains all possible USB information about a `SerialPort`
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UsbPortInfo {
    /// Vendor ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Serial number (arbitrary string)
    pub serial_number: Option<String>,
    /// Manufacturer (arbitrary string)
    pub manufacturer: Option<String>,
    /// Product name (arbitrary string)
    pub product: Option<String>,
    /// The interface index of the USB serial port. This can be either the interface number of
    /// the communication interface (as is the case on Windows and Linux) or the data
    /// interface (as is the case on macOS), so you should recognize both interface numbers.
    #[cfg(feature = "usbportinfo-interface")]
    pub interface: Option<u8>,
}

/// The physical type of a `SerialPort`
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SerialPortType {
    /// The serial port is connected via USB
    UsbPort(UsbPortInfo),
    /// The serial port is connected via PCI (permanent port)
    PciPort,
    /// The serial port is connected via Bluetooth
    BluetoothPort,
    /// It can't be determined how the serial port is connected
    Unknown,
}

/// A device-independent implementation of serial port information
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SerialPortInfo {
    /// The short name of the serial port
    pub port_name: String,
    /// The hardware device type that exposes this port
    pub port_type: SerialPortType,
}

/// Construct a builder of `SerialPort` objects
///
/// `SerialPort` objects are built using the Builder pattern through the `new` function. The
/// resultant `SerialPortBuilder` object can be copied, reconfigured, and saved making working with
/// multiple serial ports a little easier.
///
/// To open a new serial port:
/// ```no_run
/// serialport::new("/dev/ttyUSB0", 9600).open().expect("Failed to open port");
/// ```
pub fn new<'a>(path: impl Into<std::borrow::Cow<'a, str>>, baud_rate: u32) -> SerialPortBuilder {
    SerialPortBuilder {
        path: path.into().into_owned(),
        baud_rate,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(0),
    }
}

/// Returns a list of all serial ports on system
///
/// It is not guaranteed that these ports exist or are available even if they're
/// returned by this function.
pub fn available_ports() -> Result<Vec<SerialPortInfo>> {
    #[cfg(unix)]
    return crate::posix::available_ports();

    #[cfg(windows)]
    return crate::windows::available_ports();

    #[cfg(not(any(unix, windows)))]
    Err(Error::new(
        ErrorKind::Unknown,
        "available_ports() not implemented for platform",
    ))
}
