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

#[macro_export]
macro_rules! compose_edge {
    (
        $vis:vis struct $name:ident {
            $( $field:ident : $store:ty ),* $(,)?
        }
    ) => {
        #[derive(Debug, Default)]
        $vis struct $name {
            $( $field: $store, )*
        }

        $(
            impl $crate::engine::tree::components::HasComponent<$store> for $name {
                #[inline]
                fn component(&self) -> &$store {
                    &self.$field
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! compose_node {
    (
        $vis:vis struct $name:ident {
            $( $field:ident : $store:ty ),* $(,)?
        }
    ) => {
        #[derive(Debug, Default)]
        $vis struct $name<E: $crate::engine::tree::components::EdgeType = $crate::engine::tree::edges::AvgScoreEdge> {
            $( $field: $store, )*
            edges: $crate::engine::tree::EdgeRangeStore,
            _edge: ::std::marker::PhantomData<fn() -> E>,
        }

        impl<E: $crate::engine::tree::components::EdgeType>
            $crate::engine::tree::components::Clear for $name<E>
        {
            #[inline]
            fn clear(&self) {
                $( $crate::engine::tree::components::Clear::clear(&self.$field); )*
                $crate::engine::tree::components::Clear::clear(&self.edges);
            }
        }

        $(
            impl<E: $crate::engine::tree::components::EdgeType>
                $crate::engine::tree::components::HasComponent<$store> for $name<E>
            {
                #[inline]
                fn component(&self) -> &$store {
                    &self.$field
                }
            }
        )*

        impl<E: $crate::engine::tree::components::EdgeType>
            $crate::engine::tree::components::NodeType for $name<E>
        {
            type Edge = E;

            #[inline]
            fn edge_range_store(&self) -> &$crate::engine::tree::EdgeRangeStore {
                &self.edges
            }
        }
    };
}
