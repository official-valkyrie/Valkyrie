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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn lerp(self, other: Rgb, amount: f32) -> Rgb {
        let mix = |from: u8, to: u8| {
            (f32::from(from) + (f32::from(to) - f32::from(from)) * amount).round() as u8
        };

        Rgb(
            mix(self.0, other.0),
            mix(self.1, other.1),
            mix(self.2, other.2),
        )
    }

    pub fn parse(hex: &str) -> Option<Rgb> {
        let value = u32::from_str_radix(hex, 16).ok()?;

        (hex.len() == 6).then_some(Rgb((value >> 16) as u8, (value >> 8) as u8, value as u8))
    }
}

pub const BACKGROUND: Rgb = Rgb(8, 10, 16);
pub const SURFACE: Rgb = Rgb(20, 26, 40);
pub const HOVER: Rgb = Rgb(28, 36, 56);

pub const NAVY: Rgb = Rgb(30, 52, 92);
pub const GOLD: Rgb = Rgb(201, 162, 98);
pub const GOLD_LIGHT: Rgb = Rgb(230, 202, 150);
pub const WHITE: Rgb = Rgb(236, 236, 230);
pub const TEXT: Rgb = Rgb(196, 202, 212);
pub const STEEL: Rgb = Rgb(132, 144, 166);
pub const MUTED: Rgb = Rgb(88, 98, 118);
pub const RULE: Rgb = Rgb(36, 46, 66);
pub const ERROR: Rgb = Rgb(222, 108, 96);

pub const LIGHT_SQUARE: Rgb = Rgb(56, 76, 114);
pub const DARK_SQUARE: Rgb = Rgb(32, 46, 76);
