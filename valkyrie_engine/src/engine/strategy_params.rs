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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Spin,
    Check,
    String,
    Combo,
    Button,
}

impl OptionKind {
    pub const fn uci_name(self) -> &'static str {
        match self {
            Self::Spin => "spin",
            Self::Check => "check",
            Self::String => "string",
            Self::Combo => "combo",
            Self::Button => "button",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSpec {
    pub name: String,
    pub kind: OptionKind,
    pub default: String,
    pub range: Option<(String, String)>,
    pub variants: &'static [&'static str],
    pub tunable: bool,
}

pub trait UciOptionType {
    const KIND: OptionKind;
    const TUNE_KIND: &'static str;
    const VARIANTS: &'static [&'static str] = &[];
}

impl UciOptionType for bool {
    const KIND: OptionKind = OptionKind::Check;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for i32 {
    const KIND: OptionKind = OptionKind::Spin;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for i64 {
    const KIND: OptionKind = OptionKind::Spin;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for u32 {
    const KIND: OptionKind = OptionKind::Spin;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for u64 {
    const KIND: OptionKind = OptionKind::Spin;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for usize {
    const KIND: OptionKind = OptionKind::Spin;
    const TUNE_KIND: &'static str = "int";
}

impl UciOptionType for f32 {
    const KIND: OptionKind = OptionKind::String;
    const TUNE_KIND: &'static str = "float";
}

impl UciOptionType for f64 {
    const KIND: OptionKind = OptionKind::String;
    const TUNE_KIND: &'static str = "float";
}

impl UciOptionType for String {
    const KIND: OptionKind = OptionKind::String;
    const TUNE_KIND: &'static str = "string";
}

pub trait StrategyParams: std::fmt::Debug + Clone + Send + Sync {
    const OPTION_NAMES: OptionNames = OptionNames {
        own: &[],
        inner: &[],
    };

    fn new() -> Self;
    fn set_option(&mut self, name: &str, value: &str) -> Result<(), OptionViolation>;
    fn visit_options(&self, visit: &dyn Fn(&OptionSpec));
    fn visit_tunable_options(&self, visit: &dyn Fn(&OptionSpec));
    fn print_tunables(&self, print: &dyn Fn(&str));
}

const DUPLICATE_MESSAGE: &str = "Duplicate option name: ";
const MESSAGE_CAPACITY: usize = 128;

#[doc(hidden)]
#[derive(Debug)]
pub struct OptionNames {
    pub own: &'static [&'static str],
    pub inner: &'static [&'static OptionNames],
}

impl OptionNames {
    pub const fn assert_unique(&self) {
        let Some(name) = self.find_duplicate(self) else {
            return;
        };

        let mut message = [0u8; MESSAGE_CAPACITY];
        let len = write_message(name, &mut message);
        let (bytes, _) = message.split_at(len);

        match std::str::from_utf8(bytes) {
            Ok(text) => panic!("{}", text),
            Err(_) => panic!("Duplicate option name"),
        }
    }

    const fn find_duplicate(&self, root: &OptionNames) -> Option<&'static str> {
        let mut idx = 0;
        while idx < self.own.len() {
            if root.count(self.own[idx]) > 1 {
                return Some(self.own[idx]);
            }
            idx += 1;
        }

        let mut idx = 0;
        while idx < self.inner.len() {
            if let Some(name) = self.inner[idx].find_duplicate(root) {
                return Some(name);
            }
            idx += 1;
        }

        None
    }

    const fn count(&self, name: &str) -> usize {
        let mut total = 0;

        let mut idx = 0;
        while idx < self.own.len() {
            if self.own[idx].eq_ignore_ascii_case(name) {
                total += 1;
            }
            idx += 1;
        }

        let mut idx = 0;
        while idx < self.inner.len() {
            total += self.inner[idx].count(name);
            idx += 1;
        }

        total
    }
}

const fn write_message(name: &str, message: &mut [u8; MESSAGE_CAPACITY]) -> usize {
    let mut len = 0;

    let mut idx = 0;
    while idx < DUPLICATE_MESSAGE.len() {
        message[len] = DUPLICATE_MESSAGE.as_bytes()[idx];
        len += 1;
        idx += 1;
    }

    let mut idx = 0;
    while idx < name.len() && len < MESSAGE_CAPACITY {
        message[len] = name.as_bytes()[idx];
        len += 1;
        idx += 1;
    }

    len
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

    fn visit_options(&self, _visit: &dyn Fn(&OptionSpec)) {}

    fn visit_tunable_options(&self, _visit: &dyn Fn(&OptionSpec)) {}

    fn print_tunables(&self, _print: &dyn Fn(&str)) {}
}
