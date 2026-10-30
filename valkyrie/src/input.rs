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
    collections::VecDeque,
    io::stdin,
    sync::mpsc::{Receiver, RecvTimeoutError, channel},
    thread,
    time::Duration,
};

pub enum Poll {
    Line(String),
    Idle,
    Closed,
}

pub struct Input {
    receiver: Receiver<String>,
    queue: VecDeque<String>,
}

impl Input {
    pub fn spawn() -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            let mut line = String::new();

            while matches!(stdin().read_line(&mut line), Ok(read) if read > 0) {
                if sender.send(line.trim().to_string()).is_err() {
                    break;
                }

                line.clear();
            }
        });

        Self {
            receiver,
            queue: VecDeque::new(),
        }
    }

    pub fn next(&mut self) -> Option<String> {
        self.queue.pop_front().or_else(|| self.receiver.recv().ok())
    }

    pub fn poll(&mut self, timeout: Duration) -> Poll {
        match self.receiver.recv_timeout(timeout) {
            Ok(line) => Poll::Line(line),
            Err(RecvTimeoutError::Timeout) => Poll::Idle,
            Err(RecvTimeoutError::Disconnected) => {
                thread::sleep(timeout);
                Poll::Closed
            }
        }
    }

    pub fn has_queued(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn push_back(&mut self, line: String) {
        self.queue.push_back(line);
    }
}
