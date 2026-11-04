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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug)]
pub struct Hsl {
    h: f32,
    s: f32,
    l: f32,
}

impl Hsl {
    pub fn from_rgb(rgb: Rgb) -> Self {
        let r = rgb.0 as f32 / 255.0;
        let g = rgb.1 as f32 / 255.0;
        let b = rgb.2 as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f32::EPSILON {
            return Hsl { h: 0.0, s: 0.0, l };
        }

        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };

        let h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } * 60.0;

        Hsl { h, s, l }
    }

    pub fn to_rgb(self) -> Rgb {
        let h = self.h / 360.0;
        let s = self.s;
        let l = self.l;

        let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 0.5 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        };

        if s == 0.0 {
            let v = (l * 255.0).round() as u8;
            return Rgb(v, v, v);
        }

        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        Rgb(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }

    pub fn lerp(self, other: Hsl, t: f32) -> Hsl {
        let t = t.clamp(0.0, 1.0);
        let mut dh = other.h - self.h;
        if dh > 180.0 { dh -= 360.0; }
        if dh < -180.0 { dh += 360.0; }

        Hsl {
            h: (self.h + dh * t).rem_euclid(360.0),
            s: self.s + (other.s - self.s) * t,
            l: self.l + (other.l - self.l) * t,
        }
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

pub const WINNING: Rgb = Rgb(96, 186, 74);
pub const BALANCED: Rgb = Rgb(247, 247, 104);
pub const LOSING: Rgb = Rgb(214, 86, 64);

pub const LIGHT_SQUARE: Rgb = Rgb(56, 76, 114);
pub const DARK_SQUARE: Rgb = Rgb(32, 46, 76);
pub const PIECE_OUTLINE: Rgb = Rgb(10, 13, 20);
