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
macro_rules! define_strategy_params {
    (
        $name:ident $(< $( $gen:ident : $bound:path ),* >)? {
            $(Strategies {
                $( $inner_field:ident : $inner_type:ty; )*
            })?
            $(Options {
                $(
                    [$option_key:literal] $option:ident : $option_ty:ty =>
                        $option_default:expr $(, $option_min:expr, $option_max:expr)?;
                )*
            })?
            $(Buttons {
                $(
                    $button_key:literal,
                )*
            })?
            $(Tunables {
                $(
                    $(#[$tune_meta:meta])*
                    $tunable:ident : $tunable_ty:ty =>
                        $tunable_default:expr,
                        $tunable_min:expr,
                        $tunable_max:expr,
                        $tunable_c:expr,
                        $tunable_r:expr;
                )*
            })?
            $(Variables {
                $(
                    $(#[$var_meta:meta])*
                    $variable:ident : $variable_ty:ty =
                        $variable_default:expr;
                )*
            })?
        }
    ) => {
        #[derive(Debug, Clone)]
        #[allow(non_snake_case)]
        pub struct $name $(< $( $gen: $bound ),* >)? {
            $(
                $($inner_field: $inner_type,)*
            )?
            $(
                $($option: $option_ty,)*
            )?
            $(
                $(
                #[cfg(not(feature = "release"))]
                $tunable: $tunable_ty,
                )*
            )?
            $(
                $($variable: $variable_ty,)*
            )?
        }

        impl $(< $( $gen: $bound ),* >)? Default for $name $(< $( $gen ),* >)? {
            fn default() -> Self {
                Self::new()
            }
        }

        #[allow(non_snake_case)]
        impl $(< $( $gen: $bound ),* >)? $name $(< $( $gen ),* >)? {
            pub fn new() -> Self {
                const { <Self as $crate::StrategyParams>::OPTION_NAMES.assert_unique() }

                Self {
                    $(
                        $( $inner_field: <$inner_type as $crate::StrategyParams>::new(),)*
                    )?
                    $(
                        $( $option: $option_default,)*
                    )?
                    $(
                        $(
                        #[cfg(not(feature = "release"))]
                        $tunable: $tunable_default,
                        )*
                    )?
                    $(
                        $( $variable: $variable_default,)*
                    )?
                }
            }

            $(
                $(
                    pub fn $inner_field(&self) -> &$inner_type {
                        &self.$inner_field
                    }
                )*
            )?

            $( $crate::__params_option_accessors! {
                $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
            } )?

            $( $crate::__params_tunable_accessors! {
                $( $(#[$tune_meta])* $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
            } )?

            $( $crate::__params_variable_accessors! {
                $( $(#[$var_meta])* $variable : $variable_ty = $variable_default; )*
            } )?
        }

        impl $(< $( $gen: $bound ),* >)? $crate::StrategyParams for $name $(< $( $gen ),* >)? {
            const OPTION_NAMES: $crate::OptionNames = $crate::OptionNames {
                own: &[
                    $( $( $option_key, )* )?
                    $( $( $button_key, )* )?
                    $( $( stringify!($tunable), )* )?
                    $( $( stringify!($variable), )* )?
                ],
                inner: &[
                    $( $( &<$inner_type as $crate::StrategyParams>::OPTION_NAMES, )* )?
                ],
            };

            fn new() -> Self {
                Self::new()
            }

            fn set_option(&mut self, _name: &str, _value: &str) -> ::std::result::Result<(), $crate::OptionViolation> {
                $( $crate::__params_set_options!(self, _name, _value;
                    $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
                ); )?

                $( $crate::__params_check_buttons!(_name; $( $button_key, )*); )?

                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_set_tunables!(self, _name, _value;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $( $crate::__params_set_variables!(self, _name, _value;
                    $( $variable : $variable_ty = $variable_default; )*
                ); )?

                $(
                    $(
                        match $crate::StrategyParams::set_option(&mut self.$inner_field, _name, _value) {
                            Err($crate::OptionViolation::Unknown(_)) => {}
                            result => return result,
                        }
                    )*
                )?

                Err($crate::OptionViolation::Unknown(_name.to_string()))
            }

            fn print_options(&self, _print: &dyn Fn(&str)) {
                $( $crate::__params_print_options!(self, _print;
                    $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
                ); )?

                $( $crate::__params_print_buttons!(_print; $( $button_key, )*); )?

                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_print_tunable_options!(self, _print;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $(
                    $( $crate::StrategyParams::print_options(&self.$inner_field, _print); )*
                )?
            }

            fn print_tunables(&self, _print: &dyn Fn(&str)) {
                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_print_tunables!(@live self, _print;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }
                #[cfg(feature = "release")]
                {
                    $( $crate::__params_print_tunables!(@default _print;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $(
                    $( $crate::StrategyParams::print_tunables(&self.$inner_field, _print); )*
                )?
            }
        }
    };
}
