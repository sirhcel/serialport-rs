mod config;

use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

fn wait_for_file<P: AsRef<Path>>(path: P, timeout: Duration) -> Result<(), ()> {
    let end = SystemTime::now() + timeout;

    loop {
        if path.as_ref().exists() {
            return Ok(());
        }

        if SystemTime::now() > end {
            return Err(());
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Check that a pseudo terminal can be opened without an error even in case the baud rate is not
/// set to zero.
#[test]
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), ignore)]
fn test_open_pty_posix() {
    // Create temp dir and pseudo terminal paths.
    let tmp_dir = TempDir::new().unwrap();
    let our_pty = tmp_dir.path().join("ttyV0");
    let other_pty = tmp_dir.path().join("ttyV1");

    // Create pseudo terminal pair and wait for it to become available.
    let mut socat = Command::new("socat")
        .args([
            format!("PTY,link={},raw,echo=0,mode=700", our_pty.to_str().unwrap()),
            format!(
                "PTY,link={},raw,echo=0,mode=700",
                other_pty.to_str().unwrap()
            ),
        ])
        .spawn()
        .unwrap();
    wait_for_file(&our_pty, Duration::from_millis(1000)).unwrap();

    // Check that one of the pseudo terminals can be opened successfully and that there is no error
    // like seen in https://github.com/serialport/serialport-rs/issues/262.
    let _port = serialport::new(other_pty.to_str().unwrap(), 115200)
        .open()
        .unwrap();

    // Release pseudo terminal pair (by terminating socat).
    socat.kill().unwrap();
    socat.wait().unwrap();
}
