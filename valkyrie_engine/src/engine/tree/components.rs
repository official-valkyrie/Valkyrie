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

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

mod child;
mod draw_chance;
mod game_state;
mod mv;
mod policy;
mod q;
mod score_sum;
mod visits;

pub use child::{ChildStore, HasChild};
pub use draw_chance::{DrawStore, HasDrawChance};
pub use game_state::{GameStateStore, HasGameState};
pub use mv::{HasMove, MoveStore};
pub use policy::{HasPolicy, PolicyStore};
pub use q::HasQ;
pub use score_sum::{HasScoreSum, ScoreSumStore};
pub use visits::{HasVisits, VisitsStore};

pub trait HasComponent<C> {
    fn component(&self) -> &C;
}

impl<C> HasComponent<C> for C {
    #[inline]
    fn component(&self) -> &C {
        self
    }
}

pub trait Clear {
    fn clear(&self);
}

pub trait EdgeType: Clone + Default + std::fmt::Debug + Send + Sync + 'static + HasMove + HasChild {}
impl<T: Clone + Default + std::fmt::Debug + Send + Sync + 'static + HasMove + HasChild> EdgeType for T {}

pub trait NodeType: Clone + Default + std::fmt::Debug + Send + Sync + 'static + HasGameState + Clear {
    type Edge: EdgeType;

    fn edges(&self) -> RwLockReadGuard<'_, Vec<Self::Edge>>;
    fn edges_mut(&self) -> RwLockWriteGuard<'_, Vec<Self::Edge>>;

    #[inline]
    fn edge_count(&self) -> usize {
        self.edges().len()
    }
}
