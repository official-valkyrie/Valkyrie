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

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::components::{Clear, EdgeType};

#[derive(Debug, Default)]
pub struct EdgesStore<E: EdgeType>(RwLock<Vec<E>>);

impl<E: EdgeType> Clear for EdgesStore<E> {
    #[inline]
    fn clear(&self) {
        self.write().clear();
    }
}

impl<E: EdgeType> EdgesStore<E> {
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<'_, Vec<E>> {
        self.0.read().unwrap()
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<'_, Vec<E>> {
        self.0.write().unwrap()
    }
}

#[macro_export]
macro_rules! connect_edges {
    ($name:ident) => {
        impl<E: $crate::engine::tree::components::EdgeType>
            $crate::engine::tree::components::NodeType for $name<E>
        {
            type Edge = E;

            #[inline]
            fn edges(&self) -> ::std::sync::RwLockReadGuard<'_, ::std::vec::Vec<E>> {
                self.edges.read()
            }

            #[inline]
            fn edges_mut(&self) -> ::std::sync::RwLockWriteGuard<'_, ::std::vec::Vec<E>> {
                self.edges.write()
            }
        }
    };
}
