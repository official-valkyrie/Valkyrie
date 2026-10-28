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
pub enum OptionViolation {
    MissingName,
    Unknown(String),
    MissingValue {
        name: String,
    },
    InvalidValue {
        name: String,
        value: String,
    },
    OutOfRange {
        name: String,
        value: String,
        min: String,
        max: String,
    },
    Unchanged {
        name: String,
        value: String,
    },
}

impl std::fmt::Display for OptionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingName => write!(f, "Missing option name"),
            Self::Unknown(name) => write!(f, "Unknown option '{name}'"),
            Self::MissingValue { name } => write!(f, "Missing value for {name}"),
            Self::InvalidValue { name, value } => {
                write!(f, "Incorrect value '{value}' for {name}")
            }
            Self::OutOfRange {
                name,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Value '{value}' out of range for {name}, expected {min} to {max}"
                )
            }
            Self::Unchanged { name, value } => {
                write!(f, "Value of {name} is already {value}")
            }
        }
    }
}

impl std::error::Error for OptionViolation {}

pub trait UciOptionType {
    const UCI_TYPE: &'static str;
    const TUNE_KIND: &'static str;
}

impl UciOptionType for bool {
    const UCI_TYPE: &'static str = "check";
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for i32 {
    const UCI_TYPE: &'static str = "spin";
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for i64 {
    const UCI_TYPE: &'static str = "spin";
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for u32 {
    const UCI_TYPE: &'static str = "spin";
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for u64 {
    const UCI_TYPE: &'static str = "spin";
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for f32 {
    const UCI_TYPE: &'static str = "string";
    const TUNE_KIND: &'static str = "float";
}

impl UciOptionType for f64 {
    const UCI_TYPE: &'static str = "string";
    const TUNE_KIND: &'static str = "float";
}

impl UciOptionType for String {
    const UCI_TYPE: &'static str = "string";
    const TUNE_KIND: &'static str = "string";
}

pub trait StrategyParams: std::fmt::Debug + Clone + Send + Sync {
    fn new() -> Self;
    fn set_option(&mut self, name: &str, value: &str) -> Result<(), OptionViolation>;
    fn print_options(&self);
    fn print_tunables(&self);
}

#[derive(Debug, Clone)]
pub struct EmptyParams;

impl StrategyParams for EmptyParams {
    fn new() -> Self {
        Self
    }

    fn set_option(&mut self, name: &str, _value: &str) -> Result<(), OptionViolation> {
        Err(OptionViolation::Unknown(name.to_string()))
    }

    fn print_options(&self) {}

    fn print_tunables(&self) {}
}
