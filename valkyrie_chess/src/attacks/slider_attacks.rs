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

use std::mem;

pub static LOOKUP: &[LookupParams] = unsafe {
    let bytes = &LOOKUP_RAW.0;
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const LookupParams,
        bytes.len() / mem::size_of::<LookupParams>(),
    )
};

pub static ATTACK_TABLE: &[u64] = unsafe {
    let bytes = &ATTACKS_RAW.0;
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const u64,
        bytes.len() / mem::size_of::<u64>(),
    )
};

#[repr(C, align(8))]
struct AlignedData<T: ?Sized>(T);

#[cfg(target_feature = "bmi2")]
static LOOKUP_RAW: &AlignedData<[u8]> =
    &AlignedData(*include_bytes!("slider_tables/pext_lookups.table"));

#[cfg(target_feature = "bmi2")]
static ATTACKS_RAW: &AlignedData<[u8]> =
    &AlignedData(*include_bytes!("slider_tables/pext_attacks.table"));

#[cfg(not(target_feature = "bmi2"))]
static LOOKUP_RAW: &AlignedData<[u8]> =
    &AlignedData(*include_bytes!("slider_tables/plain_lookups.table"));

#[cfg(not(target_feature = "bmi2"))]
static ATTACKS_RAW: &AlignedData<[u8]> =
    &AlignedData(*include_bytes!("slider_tables/plain_attacks.table"));

#[cfg(target_feature = "bmi2")]
#[repr(C)]
pub struct LookupParams {
    mask: u64,
    pub offset: u32,
}

#[cfg(not(target_feature = "bmi2"))]
#[repr(C)]
pub struct LookupParams {
    magic: u64,
    mask: u64,
    pub offset: u32,
    shift: u8,
}

#[inline(always)]
#[cfg(target_feature = "bmi2")]
pub fn calculate_index(occupancy: u64, params: &LookupParams) -> usize {
    unsafe { core::arch::x86_64::_pext_u64(occupancy, params.mask) as usize }
}

#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
pub const fn calculate_index(occupancy: u64, params: &LookupParams) -> usize {
    (occupancy | !params.mask)
        .wrapping_mul(params.magic)
        .wrapping_shr(params.shift as u32) as usize
}
