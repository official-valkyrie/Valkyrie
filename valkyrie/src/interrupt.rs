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

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

static SEARCHING: AtomicBool = AtomicBool::new(false);
static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);
static SWALLOWED: AtomicBool = AtomicBool::new(false);

const SWALLOW_GRACE: Duration = Duration::from_millis(100);
const SWALLOW_CHECK: Duration = Duration::from_millis(10);

pub fn install() {
    platform::install();
}

pub fn set_searching(searching: bool) {
    STOP_REQUESTED.store(false, Ordering::SeqCst);
    SEARCHING.store(searching, Ordering::SeqCst);
}

pub fn take_stop_request() -> bool {
    STOP_REQUESTED.swap(false, Ordering::SeqCst)
}

pub fn was_swallowed() -> bool {
    let mut waited = Duration::ZERO;

    loop {
        if SWALLOWED.swap(false, Ordering::SeqCst) {
            return true;
        }

        if waited >= SWALLOW_GRACE {
            return false;
        }

        thread::sleep(SWALLOW_CHECK);
        waited += SWALLOW_CHECK;
    }
}

fn swallow_if_searching() -> bool {
    if !SEARCHING.load(Ordering::SeqCst) {
        return false;
    }

    STOP_REQUESTED.store(true, Ordering::SeqCst);
    SWALLOWED.store(true, Ordering::SeqCst);
    true
}

#[cfg(windows)]
mod platform {
    const CTRL_C_EVENT: u32 = 0;

    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    unsafe extern "system" fn handler(event: u32) -> i32 {
        i32::from(event == CTRL_C_EVENT && super::swallow_if_searching())
    }

    pub fn install() {
        unsafe {
            SetConsoleCtrlHandler(None, 0);
            SetConsoleCtrlHandler(Some(handler), 1);
        }
    }
}

#[cfg(unix)]
mod platform {
    const SIGINT: i32 = 2;
    const INTERRUPTED_EXIT_CODE: i32 = 130;

    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
        fn _exit(status: i32) -> !;
    }

    extern "C" fn handler(_signal: i32) {
        if !super::swallow_if_searching() {
            unsafe { _exit(INTERRUPTED_EXIT_CODE) }
        }
    }

    pub fn install() {
        unsafe {
            signal(SIGINT, handler as extern "C" fn(i32) as usize);
        }
    }
}

#[cfg(not(any(windows, unix)))]
mod platform {
    pub fn install() {}
}
