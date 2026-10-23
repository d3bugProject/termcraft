use std::sync::mpsc::{channel, Receiver};
use std::thread;

use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows_sys::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
    PROCESS_INFORMATION, STARTUPINFOEXW,
};

// PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE value:
// (0x00020016) = ProcThreadAttributeValue(22, FALSE, TRUE, FALSE)
const PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE: usize = 0x00020016;

pub struct WinConPty {
    hpcon: HPCON,
    input_write: HANDLE,
    output_read: HANDLE,
    process_info: PROCESS_INFORMATION,
    rx: Receiver<Vec<u8>>,
}

unsafe impl Send for WinConPty {}
unsafe impl Sync for WinConPty {}

impl WinConPty {
    pub fn spawn(cols: i16, rows: i16, command: &str) -> Result<Self, String> {
        unsafe {
            // 1. Create pipes for PseudoConsole communication
            let mut input_read: HANDLE = 0;
            let mut input_write: HANDLE = 0;
            let mut output_read: HANDLE = 0;
            let mut output_write: HANDLE = 0;

            let mut sa = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: std::ptr::null_mut(),
                bInheritHandle: 1,
            };

            if CreatePipe(&mut input_read, &mut input_write, &mut sa, 0) == 0 {
                return Err("Failed to create input pipe".to_string());
            }

            if CreatePipe(&mut output_read, &mut output_write, &mut sa, 0) == 0 {
                CloseHandle(input_read);
                CloseHandle(input_write);
                return Err("Failed to create output pipe".to_string());
            }

            // 2. Create the PseudoConsole
            let size = COORD { X: cols, Y: rows };
            let mut hpcon: HPCON = 0;
            let hr = CreatePseudoConsole(size, input_read, output_write, 0, &mut hpcon);
            if hr != 0 {
                CloseHandle(input_read);
                CloseHandle(input_write);
                CloseHandle(output_read);
                CloseHandle(output_write);
                return Err(format!("CreatePseudoConsole failed with HRESULT: 0x{:08X}", hr));
            }

            // ConPTY owns input_read and output_write handles now
            CloseHandle(input_read);
            CloseHandle(output_write);

            // 3. Prepare startup info with ConPTY attribute
            let mut bytes_needed: usize = 0;
            InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut bytes_needed);

            let mut attr_buf = vec![0u8; bytes_needed];
            let lp_attribute_list = attr_buf.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;

            if InitializeProcThreadAttributeList(lp_attribute_list, 1, 0, &mut bytes_needed) == 0 {
                ClosePseudoConsole(hpcon);
                CloseHandle(input_write);
                CloseHandle(output_read);
                return Err("Failed to initialize proc thread attribute list".to_string());
            }

            if UpdateProcThreadAttribute(
                lp_attribute_list,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
                hpcon as _,
                std::mem::size_of::<HPCON>(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == 0
            {
                DeleteProcThreadAttributeList(lp_attribute_list);
                ClosePseudoConsole(hpcon);
                CloseHandle(input_write);
                CloseHandle(output_read);
                return Err("Failed to update proc thread attribute".to_string());
            }

            let mut si_ex: STARTUPINFOEXW = std::mem::zeroed();
            si_ex.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
            si_ex.lpAttributeList = lp_attribute_list;

            let mut pi: PROCESS_INFORMATION = std::mem::zeroed();

            // Prepare command line wide string
            let mut cmd_wide: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();

            let success = CreateProcessW(
                std::ptr::null(),
                cmd_wide.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                EXTENDED_STARTUPINFO_PRESENT,
                std::ptr::null(),
                std::ptr::null(),
                &si_ex.StartupInfo,
                &mut pi,
            );

            DeleteProcThreadAttributeList(lp_attribute_list);

            if success == 0 {
                ClosePseudoConsole(hpcon);
                CloseHandle(input_write);
                CloseHandle(output_read);
                return Err("Failed to spawn process in ConPTY".to_string());
            }

            // Background reader thread
            let (tx, rx) = channel::<Vec<u8>>();
            let reader_handle = output_read;
            thread::spawn(move || {
                let mut buffer = [0u8; 4096];
                loop {
                    let mut bytes_read: u32 = 0;
                    let ok = ReadFile(
                        reader_handle,
                        buffer.as_mut_ptr(),
                        buffer.len() as u32,
                        &mut bytes_read,
                        std::ptr::null_mut(),
                    );
                    if ok == 0 || bytes_read == 0 {
                        break;
                    }
                    if tx.send(buffer[..bytes_read as usize].to_vec()).is_err() {
                        break;
                    }
                }
            });

            Ok(Self {
                hpcon,
                input_write,
                output_read,
                process_info: pi,
                rx,
            })
        }
    }

    pub fn read_bytes(&self) -> Option<Vec<u8>> {
        self.rx.try_recv().ok()
    }

    pub fn write_bytes(&self, bytes: &[u8]) {
        unsafe {
            let mut written: u32 = 0;
            WriteFile(
                self.input_write,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn resize(&self, cols: i16, rows: i16) {
        unsafe {
            let size = COORD { X: cols, Y: rows };
            ResizePseudoConsole(self.hpcon, size);
        }
    }
}

impl Drop for WinConPty {
    fn drop(&mut self) {
        unsafe {
            if self.hpcon != 0 {
                ClosePseudoConsole(self.hpcon);
            }
            if self.input_write != 0 && self.input_write != INVALID_HANDLE_VALUE {
                CloseHandle(self.input_write);
            }
            if self.output_read != 0 && self.output_read != INVALID_HANDLE_VALUE {
                CloseHandle(self.output_read);
            }
            if self.process_info.hProcess != 0 && self.process_info.hProcess != INVALID_HANDLE_VALUE {
                CloseHandle(self.process_info.hProcess);
            }
            if self.process_info.hThread != 0 && self.process_info.hThread != INVALID_HANDLE_VALUE {
                CloseHandle(self.process_info.hThread);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_conpty() {
        let pty = WinConPty::spawn(80, 24, "cmd.exe");
        if let Err(ref e) = pty {
            println!("ConPTY spawn error: {}", e);
        }
        assert!(pty.is_ok());
    }
}
