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

pub mod backpropagation_strategy;
pub mod best_move_strategy;
pub mod expansion_strategy;
pub mod exploration_strategy;
pub mod payload;
pub mod search_strategy;
pub mod simulation_strategy;
pub mod time_manager_strategy;

pub use self::backpropagation_strategy::BackpropagationStrategy;
pub use self::best_move_strategy::BestMoveStrategy;
pub use self::expansion_strategy::ExpansionStrategy;
pub use self::exploration_strategy::ExplorationStrategy;
pub use self::payload::{HasWdl, Payload};
pub use self::search_strategy::SearchStrategy;
pub use self::simulation_strategy::SimulationStrategy;
pub use self::time_manager_strategy::TimeManagerStrategy;

use crate::engine::strategy_params::StrategyParams;

pub struct Unspecified;

pub trait Strategy: std::fmt::Debug + Send + Sync + 'static {
    type Params: StrategyParams + Send + Sync;
}

crate::define_strategy_params! {
    GeneralParams {
        Options {
            ["Hash"] hash: i32 => 1024, 1, 524288;
            ["UCI_Chess960"] chess960: bool => false;
            ["ItersAsNodes"] iters_as_nodes: bool => true;
        }
        Buttons {
            "ClearHash",
        }
    }
}

crate::define_engine_config! {
    general: GeneralParams,
    strategies {
        //Type           Trait                   Field/Name        Default Value
        BestMove:        BestMoveStrategy      | best_move       | Unspecified,
        Exploration:     ExplorationStrategy   | exploration     | Unspecified,
        Expansion:       ExpansionStrategy     | expansion       | Unspecified,
        Simulation:      SimulationStrategy    | simulation      | Unspecified,
        Backpropagation: BackpropagationStrategy | backpropagation | Unspecified,
        Search:          SearchStrategy        | search          | Unspecified,
        TimeManager:     TimeManagerStrategy   | time_manager    | Unspecified,
    }
}
