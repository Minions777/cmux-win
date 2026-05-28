use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};

/// Handle to a PTY (pseudo-terminal) process
pub struct PtyHandle {
    child: Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
    buffer: Vec<u8>,
}

impl PtyHandle {
    /// Create a new PTY with the given shell command
    pub fn new(shell: &str, cwd: &str) -> io::Result<Self> {
        let mut cmd = Command::new(shell);
        cmd.current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Windows, enable VT processing
        #[cfg(windows)]
        {
            use windows::Win32::System::Console::*;
            unsafe {
                // Enable ANSI/VT processing on stdout
n                if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
                    let mut mode = CONSOLE_MODE::default();
                    if GetConsoleMode(handle, &mut mode).is_ok() {
                        let _ = SetConsoleMode(
                            handle,
                            mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                        );
                    }
                }
                // Enable ANSI/VT processing on stderr
                if let Ok(handle) = GetStdHandle(STD_ERROR_HANDLE) {
                    let mut mode = CONSOLE_MODE::default();
                    if GetConsoleMode(handle, &mut mode).is_ok() {
                        let _ = SetConsoleMode(
                            handle,
                            mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                        );
                    }
                }
            }
        }

        let mut child = cmd.spawn()?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        // Set stdout to non-blocking mode
        #[cfg(unix)]
        if let Some(ref stdout) = stdout {
            use std::os::unix::io::AsRawFd;
            let fd = stdout.as_raw_fd();
            unsafe {
                libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK);
            }
        }

        Ok(Self {
            child,
            stdin,
            stdout,
            buffer: vec![0u8; 65536],
        })
    }

    /// Write data to the terminal's stdin
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(ref mut stdin) = self.stdin {
            stdin.write_all(data)?;
            stdin.flush()?;
        }
        Ok(())
    }

    /// Read available data from stdout (non-blocking)
    pub fn read_available(&mut self) -> Vec<u8> {
        let mut result = Vec::new();

        if let Some(ref mut stdout) = self.stdout {
            loop {
                match stdout.read(&mut self.buffer) {
                    Ok(0) => break,
                    Ok(n) => result.extend_from_slice(&self.buffer[..n]),
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }

        result
n    }

    /// Resize the terminal (ConPTY on Windows, ioctl on Unix)
    pub fn resize(&mut self, _cols: u16, _rows: u16) -> io::Result<()> {
        // TODO: Implement resize for ConPTY and Unix PTY
        // On Windows with ConPTY: ResizePseudoConsole
        // On Unix: ioctl TIOCSWINSZ
        Ok(())
    }

    /// Kill the child process
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}

impl Drop for PtyHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
