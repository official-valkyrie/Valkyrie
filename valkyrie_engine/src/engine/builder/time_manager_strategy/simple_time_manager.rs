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

use crate::prelude::*;

#[derive(Debug, Default)]
pub struct SimpleTimeManager {
    hard_limit: Option<u64>,
}

crate::define_strategy_params! {
    TimeManagerParams {
        Options {
            ["MoveOverhead"] move_overhead: i32 => 10, 0, 1000;
        }
    }
}

impl Strategy for SimpleTimeManager {
    type Params = TimeManagerParams;
}

impl<C: EngineConfig> TimeManagerStrategy<C> for SimpleTimeManager {
    fn new(limits: &SearchLimits, params: &Self::Params, _engine: &Engine<C>) -> Self {
        let mut time_manager = Self { hard_limit: None };

        if let Some(move_time) = limits.move_time() {
            time_manager.hard_limit = Some(move_time);
            return time_manager;
        }

        let Some(time) = limits.time() else {
            return time_manager;
        };

        let increment = limits.increment().unwrap_or_default();
        let mtg = limits
            .moves_to_go()
            .filter(|&moves_to_go| moves_to_go > 0)
            .unwrap_or(20);
        let overhead = params.move_overhead() as u64;

        let hard_limit = (time / mtg + increment / 2).min(time);
        time_manager.hard_limit = Some(hard_limit.saturating_sub(overhead));

        time_manager
    }

    fn soft_limit(&self, _time_passed: u64, _params: &Self::Params, _engine: &Engine<C>) -> bool {
        false
    }

    fn hard_limit(&self, time_passed: u64, _params: &Self::Params, _engine: &Engine<C>) -> bool {
        match self.hard_limit {
            Some(limit) => time_passed >= limit,
            None => false,
        }
    }
}
