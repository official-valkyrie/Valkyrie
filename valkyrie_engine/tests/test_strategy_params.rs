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

use valkyrie_engine::{OptionViolation, StrategyParams};

valkyrie_engine::define_strategy_params! {
    InnerParams {
        Options {
            ["InnerOption"] inner_option: i32 => 5, 0, 10;
        }
        Buttons {
            "InnerButton",
        }
        Variables {
            inner_variable: u32 = 1;
        }
    }
}

valkyrie_engine::define_strategy_params! {
    OuterParams {
        Strategies {
            inner: InnerParams;
        }
        Options {
            ["OuterOption"] outer_option: i32 => 1, 0, 10;
        }
        Tunables {
            outer_tunable: f32 => 1.0, 0.0, 2.0, 0.1, 0.1;
        }
    }
}

valkyrie_engine::define_strategy_params! {
    GenericParams<P: StrategyParams> {
        Strategies {
            inner: P;
        }
        Options {
            ["GenericOption"] generic_option: bool => false;
        }
    }
}

#[test]
fn sets_outer_options() {
    let mut params = OuterParams::new();

    assert_eq!(params.set_option("OuterOption", "3"), Ok(()));
    assert_eq!(params.outer_option(), 3);
    assert_eq!(params.inner().inner_option(), 5);
}

#[test]
fn forwards_options_to_inner_params() {
    let mut params = OuterParams::new();

    assert_eq!(params.set_option("InnerOption", "7"), Ok(()));
    assert_eq!(params.inner().inner_option(), 7);
    assert_eq!(params.outer_option(), 1);

    assert_eq!(params.set_option("InnerButton", ""), Ok(()));

    assert_eq!(params.set_option("inner_variable", "4"), Ok(()));
    assert_eq!(params.inner().inner_variable(), 4);
}

#[test]
fn reports_inner_violations() {
    let mut params = OuterParams::new();

    assert_eq!(
        params.set_option("InnerOption", "11"),
        Err(OptionViolation::OutOfRange {
            name: "InnerOption".to_string(),
            value: "11".to_string(),
            min: "0".to_string(),
            max: "10".to_string(),
        })
    );
    assert_eq!(
        params.set_option("InnerOption", "5"),
        Err(OptionViolation::Unchanged {
            name: "InnerOption".to_string(),
            value: "5".to_string(),
        })
    );
    assert_eq!(
        params.set_option("Missing", "1"),
        Err(OptionViolation::Unknown("Missing".to_string()))
    );
}

#[test]
fn tunables_follow_release_feature() {
    let mut params = OuterParams::new();
    let result = params.set_option("outer_tunable", "1.5");

    if cfg!(feature = "release") {
        assert_eq!(
            result,
            Err(OptionViolation::Unknown("outer_tunable".to_string()))
        );
        assert_eq!(params.outer_tunable(), 1.0);
    } else {
        assert_eq!(result, Ok(()));
        assert_eq!(params.outer_tunable(), 1.5);
    }
}

#[test]
fn forwards_options_through_generic_params() {
    let mut params = GenericParams::<InnerParams>::new();

    assert_eq!(params.set_option("GenericOption", "true"), Ok(()));
    assert!(params.generic_option());

    assert_eq!(params.set_option("InnerOption", "2"), Ok(()));
    assert_eq!(params.inner().inner_option(), 2);
}
