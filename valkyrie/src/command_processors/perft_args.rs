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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerftViolation {
    InvalidDepth(String),
    UnexpectedArgument(String),
}

impl std::fmt::Display for PerftViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDepth(depth) => write!(f, "Incorrect perft depth '{depth}'"),
            Self::UnexpectedArgument(argument) => write!(f, "Unexpected argument '{argument}'"),
        }
    }
}

pub fn parse_perft(args: &[&str]) -> Result<Option<u8>, Vec<PerftViolation>> {
    let mut violations = Vec::new();

    let depth = args.first().and_then(|depth| {
        let parsed = depth.parse::<u8>().ok();

        if parsed.is_none() {
            violations.push(PerftViolation::InvalidDepth(depth.to_string()));
        }

        parsed
    });

    violations.extend(
        args.iter()
            .skip(1)
            .map(|arg| PerftViolation::UnexpectedArgument(arg.to_string())),
    );

    if violations.is_empty() {
        Ok(depth)
    } else {
        Err(violations)
    }
}
