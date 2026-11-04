/*
    This file is part of Valkyrie.

    Copyright (C) 2026 Tomasz Jaworski and Valkyrie Contributors

    Valkyrie is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Valkyrie is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Valkyrie.  If not, see <https://gnu.org>.

    Additional terms under GNU GPL version 3 section 7

    1. Linking Exception:
    If you modify this Program, or any covered work, by linking or
    combining it with the ONNX Runtime and/or NVIDIA TensorRT libraries
    (or a modified version of those libraries), containing parts covered
    by the terms of the respective ONNX Runtime and NVIDIA license
    agreements, the licensors of this Program grant you additional
    permission to convey the resulting work.

   2. Attribution Requirement (Section 7b):
    Under Section 7(b) of the GNU General Public License, version 3,
    the licensors of this Program require that any modified or redistributed
    versions of this software must prominently preserve original author
    attributions and credit to the Valkyrie Contributors in the software
    documentation.
*/

use std::io::IsTerminal;

pub fn supports_tui() -> bool {
    std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
        && std::env::var("TERM").map_or(true, |term| term != "dumb")
        && platform::enable_ansi()
}

pub fn size() -> Option<(u16, u16)> {
    platform::size()
}

pub fn write_unlocked(text: &str) {
    platform::write_unlocked(text);
}

pub fn relaunch_in_terminal() -> bool {
    platform::relaunch_in_terminal()
}

pub fn pseudo_console() -> bool {
    platform::pseudo_console()
}

pub fn copy_to_clipboard(text: &str) -> bool {
    platform::copy_to_clipboard(text)
}

pub fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    bytes
        .chunks(3)
        .flat_map(|chunk| {
            let value = chunk.iter().enumerate().fold(0u32, |value, (index, byte)| {
                value | (u32::from(*byte) << (16 - 8 * index))
            });

            (0..4).map(move |index| {
                if index <= chunk.len() {
                    char::from(ALPHABET[((value >> (18 - 6 * index)) & 63) as usize])
                } else {
                    '='
                }
            })
        })
        .collect()
}

#[cfg(windows)]
mod platform {
    use std::io::IsTerminal;

    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    const INVALID_HANDLE_VALUE: isize = -1;

    #[repr(C)]
    #[derive(Default)]
    struct Coord {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    #[derive(Default)]
    struct SmallRect {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    #[repr(C)]
    #[derive(Default)]
    struct ScreenBufferInfo {
        size: Coord,
        cursor: Coord,
        attributes: u16,
        window: SmallRect,
        maximum_size: Coord,
    }

    unsafe extern "system" {
        fn GetStdHandle(std_handle: u32) -> isize;
        fn GetConsoleMode(handle: isize, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: isize, mode: u32) -> i32;
        fn GetConsoleScreenBufferInfo(handle: isize, info: *mut ScreenBufferInfo) -> i32;
        fn WriteFile(
            handle: isize,
            buffer: *const u8,
            length: u32,
            written: *mut u32,
            overlapped: *mut core::ffi::c_void,
        ) -> i32;
        fn WriteConsoleW(
            handle: isize,
            buffer: *const u16,
            length: u32,
            written: *mut u32,
            reserved: *mut core::ffi::c_void,
        ) -> i32;
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn OpenClipboard(owner: isize) -> i32;
        fn EmptyClipboard() -> i32;
        fn SetClipboardData(format: u32, memory: isize) -> isize;
        fn CloseClipboard() -> i32;
        fn GetClassNameW(window: isize, name: *mut u16, length: i32) -> i32;
    }

    unsafe extern "system" {
        fn GlobalAlloc(flags: u32, bytes: usize) -> isize;
        fn GlobalLock(memory: isize) -> *mut u16;
        fn GlobalUnlock(memory: isize) -> i32;
        fn GlobalFree(memory: isize) -> isize;
        fn GetConsoleWindow() -> isize;
        fn GetConsoleProcessList(processes: *mut u32, count: u32) -> u32;
    }

    const CF_UNICODETEXT: u32 = 13;
    const PSEUDO_CONSOLE_CLASS: &str = "PseudoConsoleWindow";
    const LEGACY_CONSOLE_CLASS: &str = "ConsoleWindowClass";
    const WINDOWS_TERMINAL: &str = "wt.exe";
    const CLASS_NAME_LENGTH: usize = 64;

    pub fn pseudo_console() -> bool {
        console_class().is_some_and(|class| class == PSEUDO_CONSOLE_CLASS)
    }

    pub fn relaunch_in_terminal() -> bool {
        let mut processes = [0u32; 2];
        let attached = unsafe { GetConsoleProcessList(processes.as_mut_ptr(), 2) };

        let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

        if !interactive
            || attached != 1
            || console_class().is_none_or(|class| class != LEGACY_CONSOLE_CLASS)
        {
            return false;
        }

        let Ok(executable) = std::env::current_exe() else {
            return false;
        };
        let directory = std::env::current_dir().unwrap_or_default();

        std::process::Command::new(WINDOWS_TERMINAL)
            .arg("-d")
            .arg(directory)
            .arg(executable)
            .spawn()
            .is_ok()
    }

    fn console_class() -> Option<String> {
        let mut name = [0u16; CLASS_NAME_LENGTH];

        let length = unsafe {
            let window = GetConsoleWindow();

            if window == 0 {
                return None;
            }

            GetClassNameW(window, name.as_mut_ptr(), CLASS_NAME_LENGTH as i32)
        };

        let length = usize::try_from(length).ok().filter(|length| *length > 0)?;

        Some(String::from_utf16_lossy(&name[..length]))
    }

    const GMEM_MOVEABLE: u32 = 0x0002;
    const CONSOLE_CHUNK: usize = 8192;

    pub fn copy_to_clipboard(text: &str) -> bool {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            if OpenClipboard(0) == 0 {
                return false;
            }

            let copied = EmptyClipboard() != 0 && {
                let memory = GlobalAlloc(GMEM_MOVEABLE, wide.len() * 2);
                let target = if memory == 0 {
                    core::ptr::null_mut()
                } else {
                    GlobalLock(memory)
                };

                if target.is_null() {
                    false
                } else {
                    core::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
                    GlobalUnlock(memory);

                    SetClipboardData(CF_UNICODETEXT, memory) != 0 || {
                        GlobalFree(memory);
                        false
                    }
                }
            };

            CloseClipboard();
            copied
        }
    }

    pub fn write_unlocked(text: &str) {
        let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut rest = wide.as_slice();

        while !rest.is_empty() {
            let mut length = rest.len().min(CONSOLE_CHUNK);

            if length < rest.len() && (0xD800..0xDC00).contains(&rest[length - 1]) {
                length -= 1;
            }

            let mut written = 0;
            let accepted = unsafe {
                WriteConsoleW(
                    handle,
                    rest.as_ptr(),
                    length as u32,
                    &mut written,
                    core::ptr::null_mut(),
                )
            };

            if accepted == 0 || written == 0 {
                let bytes = String::from_utf16_lossy(rest).into_bytes();

                unsafe {
                    WriteFile(
                        handle,
                        bytes.as_ptr(),
                        bytes.len() as u32,
                        &mut written,
                        core::ptr::null_mut(),
                    );
                }
                return;
            }

            rest = &rest[(written as usize).min(rest.len())..];
        }
    }

    pub fn enable_ansi() -> bool {
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode = 0;

            handle != INVALID_HANDLE_VALUE
                && handle != 0
                && GetConsoleMode(handle, &mut mode) != 0
                && (mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0
                    || SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0)
        }
    }

    pub fn size() -> Option<(u16, u16)> {
        let mut info = ScreenBufferInfo::default();

        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);

            if handle == INVALID_HANDLE_VALUE
                || handle == 0
                || GetConsoleScreenBufferInfo(handle, &mut info) == 0
            {
                return None;
            }
        }

        let width = i32::from(info.window.right) - i32::from(info.window.left) + 1;
        let height = i32::from(info.window.bottom) - i32::from(info.window.top) + 1;

        Some((u16::try_from(width).ok()?, u16::try_from(height).ok()?))
    }
}

#[cfg(unix)]
mod platform {
    const STDOUT_FD: i32 = 1;
    const TIOCGWINSZ: usize = if cfg!(any(target_os = "linux", target_os = "android")) {
        0x5413
    } else {
        0x4008_7468
    };

    #[repr(C)]
    #[derive(Default)]
    struct WindowSize {
        rows: u16,
        columns: u16,
        x_pixels: u16,
        y_pixels: u16,
    }

    unsafe extern "C" {
        fn ioctl(fd: i32, request: usize, ...) -> i32;
        fn write(fd: i32, buffer: *const u8, count: usize) -> isize;
    }

    pub fn write_unlocked(text: &str) {
        let mut rest = text.as_bytes();

        while !rest.is_empty() {
            let written = unsafe { write(STDOUT_FD, rest.as_ptr(), rest.len()) };

            if written <= 0 {
                return;
            }

            rest = &rest[written as usize..];
        }
    }

    pub fn enable_ansi() -> bool {
        true
    }

    pub fn relaunch_in_terminal() -> bool {
        false
    }

    pub fn pseudo_console() -> bool {
        false
    }

    pub fn copy_to_clipboard(text: &str) -> bool {
        write_unlocked(&format!("\x1b]52;c;{}\x07", super::base64(text.as_bytes())));
        true
    }

    pub fn size() -> Option<(u16, u16)> {
        let mut window = WindowSize::default();

        let result = unsafe { ioctl(STDOUT_FD, TIOCGWINSZ, &mut window as *mut WindowSize) };

        (result == 0 && window.columns > 0 && window.rows > 0)
            .then_some((window.columns, window.rows))
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    pub fn enable_ansi() -> bool {
        false
    }

    pub fn size() -> Option<(u16, u16)> {
        None
    }

    pub fn write_unlocked(_text: &str) {}

    pub fn relaunch_in_terminal() -> bool {
        false
    }

    pub fn pseudo_console() -> bool {
        false
    }

    pub fn copy_to_clipboard(_text: &str) -> bool {
        false
    }
}
