use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};

/// Handle to a PTY (pseudo-terminal) process
/// On Windows, uses ConPTY for proper terminal emulation
/// On Unix, uses traditional PTY via forkpty
pub struct PtyHandle {
    child: Child,
    stdin: Option<Box<dyn Write + Send>>,
    stdout: Option<Box<dyn Read + Send>>,
    buffer: Vec<u8>,
    cols: u16,
    rows: u16,
    #[cfg(windows)]
    conpty: Option<ConPtyHandle>,
}

#[cfg(windows)]
struct ConPtyHandle {
    hpc: windows::Win32::System::Console::HPCON,
    input_write: windows::Win32::Foundation::HANDLE,
    output_read: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
unsafe impl Send for ConPtyHandle {}

impl PtyHandle {
    /// Create a new PTY with the given shell command
    pub fn new(shell: &str, cwd: &str) -> io::Result<Self> {
        let cols = 80;
        let rows = 24;

        #[cfg(windows)]
        {
            Self::new_conpty(shell, cwd, cols, rows)
        }

        #[cfg(unix)]
        {
            Self::new_unix(shell, cwd, cols, rows)
        }
    }

    /// Create a ConPTY-based terminal on Windows
    #[cfg(windows)]
    fn new_conpty(shell: &str, cwd: &str, cols: u16, rows: u16) -> io::Result<Self> {
        use windows::Win32::Foundation::*;
        use windows::Win32::Security::*;
        use windows::Win32::System::Console::*;
        use windows::Win32::System::Threading::*;
        use std::mem;

        unsafe {
            // Create pipes for ConPTY input/output
            let mut input_read = INVALID_HANDLE_VALUE;
            let mut input_write = INVALID_HANDLE_VALUE;
            let mut output_read = INVALID_HANDLE_VALUE;
            let mut output_write = INVALID_HANDLE_VALUE;

            let sa = SECURITY_ATTRIBUTES {
                nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                bInheritHandle: TRUE,
                lpSecurityDescriptor: std::ptr::null_mut(),
            };

            if !CreatePipe(&mut input_read, &mut input_write, Some(&sa), 0).as_bool() {
                return Err(io::Error::last_os_error());
            }
            if !CreatePipe(&mut output_read, &mut output_write, Some(&sa), 0).as_bool() {
                CloseHandle(input_read);
                CloseHandle(input_write);
                return Err(io::Error::last_os_error());
            }

            // Create pseudo console
            let coord = COORD { X: cols as i16, Y: rows as i16 };
            let hpc = match CreatePseudoConsole(coord, input_read, output_write, 0) {
                Ok(h) => h,
                Err(e) => {
                    CloseHandle(input_read);
                    CloseHandle(input_write);
                    CloseHandle(output_read);
                    CloseHandle(output_write);
                    return Err(io::Error::new(io::ErrorKind::Other, format!("CreatePseudoConsole failed: {:?}", e)));
                }
            };

            // Close the ends we don't need
            CloseHandle(input_read);
            CloseHandle(output_write);

            // Setup startup info with pseudo console
            let mut startup_info_ex = STARTUPINFOEXW::default();
            startup_info_ex.StartupInfo.cb = mem::size_of::<STARTUPINFOEXW>() as u32;
            startup_info_ex.StartupInfo.dwFlags = STARTF_USESTDHANDLES;

            // Get required attribute list size
            let mut attr_list_size: usize = 0;
            let _ = InitializeProcThreadAttributeList(
                LPPROC_THREAD_ATTRIBUTE_LIST::default(),
                1,
                0,
                &mut attr_list_size,
            );

            let attr_list_buf = vec![0u8; attr_list_size];
            startup_info_ex.lpAttributeList =
                LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buf.as_ptr() as *mut _);

            if !InitializeProcThreadAttributeList(
                startup_info_ex.lpAttributeList,
                1,
                0,
                &mut attr_list_size,
            )
            .as_bool()
            {
                ClosePseudoConsole(hpc);
                CloseHandle(input_read);
                CloseHandle(output_read);
                return Err(io::Error::last_os_error());
            }

            if !UpdateProcThreadAttribute(
                startup_info_ex.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some(hpc.Value as *const _),
                mem::size_of::<*const std::ffi::c_void>(),
                None,
                None,
            )
            .as_bool()
            {
                DeleteProcThreadAttributeList(startup_info_ex.lpAttribute_list);
                ClosePseudoConsole(hpc);
                return Err(io::Error::last_os_error());
            }

            // Create the process
            let mut process_info = PROCESS_INFORMATION::default();
            let shell_wide: Vec<u16> = shell.encode_utf16().chain(std::iter::once(0)).collect();
            let cwd_wide: Vec<u16> = cwd.encode_utf16().chain(std::iter::once(0)).collect();

            if !CreateProcessW(
                PCWSTR(shell_wide.as_ptr()),
                PWSTR(std::ptr::null_mut()),
                None,
                None,
                FALSE,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                None,
                PCWSTR(cwd_wide.as_ptr()),
                &startup_info_ex.StartupInfo,
                &mut process_info,
            )
            .as_bool()
            {
                DeleteProcThreadAttributeList(startup_info_ex.lpAttribute_list);
                ClosePseudoConsole(hpc);
                return Err(io::Error::last_os_error());
            }

            // Clean up
            CloseHandle(process_info.hThread);
            DeleteProcThreadAttributeList(startup_info_ex.lpAttribute_list);

            let child = Child::from_raw(process_info.hProcess.0 as u32);

            // Create wrapper handles for Read/Write
            let stdin_handle = HandleWrapper(input_write);
            let stdout_handle = HandleWrapper(output_read);

            Ok(Self {
                child,
                stdin: Some(Box::new(stdin_handle)),
                stdout: Some(Box::new(stdout_handle)),
                buffer: vec![0u8; 65536],
                cols,
                rows,
                conpty: Some(ConPtyHandle {
                    hpc,
                    input_write,
                    output_read,
                }),
            })
        }
    }

    /// Create a Unix PTY
    #[cfg(unix)]
    fn new_unix(shell: &str, cwd: &str, cols: u16, rows: u16) -> io::Result<Self> {
        use nix::pty::openpty;
        use nix::unistd::{fork, ForkResult, setsid, dup2, close, execv, chdir};
        use std::ffi::CString;

        let ws = nix::pty::Winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let pty = openpty(Some(&ws)).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        match unsafe { fork().map_err(|e| io::Error::new(io::ErrorKind::Other, e))? } {
            ForkResult::Parent { child } => {
                // Parent process
                close(pty.slave).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let master_fd = pty.master;

                // Set non-blocking
                unsafe {
                    let flags = libc::fcntl(master_fd, libc::F_GETFL);
                    libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }

                let stdin = unsafe { std::fs::File::from_raw_fd(master_fd) };
                let stdout = unsafe { std::fs::File::from_raw_fd(dup2(master_fd, master_fd).unwrap_or(master_fd)) };

                Ok(Self {
                    child: unsafe { Child::from_raw(child.as_raw()) },
                    stdin: Some(Box::new(stdin.try_clone()?)),
                    stdout: Some(Box::new(stdout)),
                    buffer: vec![0u8; 65536],
                    cols,
                    rows,
                })
            }
            ForkResult::Child => {
                // Child process
                setsid().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                close(pty.master).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let slave_fd = pty.slave;
                dup2(slave_fd, 0).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                dup2(slave_fd, 1).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                dup2(slave_fd, 2).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                if slave_fd > 2 {
                    close(slave_fd).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                }

                chdir(cwd).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let shell_cstr = CString::new(shell).unwrap();
                execv(&shell_cstr, &[&shell_cstr])
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                unreachable!()
            }
        }
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
    }

    /// Resize the terminal
    pub fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        self.cols = cols;
        self.rows = rows;

        #[cfg(windows)]
        {
            if let Some(ref conpty) = self.conpty {
                unsafe {
                    let coord = windows::Win32::System::Console::COORD {
                        X: cols as i16,
                        Y: rows as i16,
                    };
                    windows::Win32::System::Console::ResizePseudoConsole(conpty.hpc, coord);
                }
            }
        }

        #[cfg(unix)]
        {
            // On Unix, we'd use ioctl TIOCSWINSZ
            // This requires the master fd which we'd need to store
        }

        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    /// Check if the child process is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }

    /// Kill the child process
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}

#[cfg(windows)]
struct HandleWrapper(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Read for HandleWrapper {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::Pipes::*;

        let mut bytes_read = 0u32;
        unsafe {
            if ReadFile(
                self.0,
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut bytes_read),
                None,
            )
            .as_bool()
            {
                Ok(bytes_read as usize)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

#[cfg(windows)]
impl Write for HandleWrapper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::Pipes::*;

        let mut bytes_written = 0u32;
        unsafe {
            if WriteFile(
                self.0,
                Some(buf.as_ptr() as *const _),
                Some(&mut bytes_written),
                None,
            )
            .as_bool()
            {
                Ok(bytes_written as usize)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for PtyHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();

        #[cfg(windows)]
        {
            if let Some(conpty) = self.conpty.take() {
                unsafe {
                    windows::Win32::System::Console::ClosePseudoConsole(conpty.hpc);
                }
            }
        }
    }
}
