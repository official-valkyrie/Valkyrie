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
macro_rules! define_compound_params {
    (
        $name:ident {
            Strategies {
                $( $inner_field:ident : $inner_type:ty; )+
            }
            $(Options {
                $(
                    [$option_key:literal] $option:ident : $option_ty:ty =>
                        $option_default:expr $(, $option_min:expr, $option_max:expr)?;
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
        }
    ) => {
        $crate::define_compound_params! {
            $name<> {
                Strategies {
                    $( $inner_field : $inner_type; )+
                }
                $(Options {
                    $(
                        [$option_key] $option : $option_ty =>
                            $option_default $(, $option_min, $option_max)?;
                    )*
                })?
                $(Tunables {
                    $(
                        $(#[$tune_meta])*
                        $tunable : $tunable_ty =>
                            $tunable_default,
                            $tunable_min,
                            $tunable_max,
                            $tunable_c,
                            $tunable_r;
                    )*
                })?
            }
        }
    };

    (
        $name:ident < $( $gen:ident : $bound:path ),* > {
            Strategies {
                $( $inner_field:ident : $inner_type:ty; )+
            }
            $(Options {
                $(
                    [$option_key:literal] $option:ident : $option_ty:ty =>
                        $option_default:expr $(, $option_min:expr, $option_max:expr)?;
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
        }
    ) => {
        #[derive(Debug)]
        #[allow(non_snake_case)]
        pub struct $name<$($gen: $bound),*> {
            $( $inner_field: $inner_type, )+
            $(
                $($option: $option_ty,)*
            )?
            $(
                $(
                #[cfg(not(feature = "release"))]
                $tunable: $tunable_ty,
                )*
            )?
        }

        impl<$($gen: $bound),*> Clone for $name<$($gen),*> {
            fn clone(&self) -> Self {
                Self {
                    $( $inner_field: self.$inner_field.clone(), )+
                    $(
                        $($option: self.$option.clone(),)*
                    )?
                    $(
                        $(
                        #[cfg(not(feature = "release"))]
                        $tunable: self.$tunable.clone(),
                        )*
                    )?
                }
            }
        }

        #[allow(non_snake_case)]
        impl<$($gen: $bound),*> $name<$($gen),*> {
            $( pub fn $inner_field(&self) -> &$inner_type { &self.$inner_field } )+

            $( $crate::__params_option_accessors! {
                $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
            } )?

            $( $crate::__params_tunable_accessors! {
                $( $(#[$tune_meta])* $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
            } )?
        }

        impl<$($gen: $bound),*> $crate::StrategyParams for $name<$($gen),*> {
            fn new() -> Self {
                Self {
                    $( $inner_field: <$inner_type as $crate::StrategyParams>::new(), )+
                    $(
                        $($option: $option_default,)*
                    )?
                    $(
                        $(
                        #[cfg(not(feature = "release"))]
                        $tunable: $tunable_default,
                        )*
                    )?
                }
            }

            fn set_option(&mut self, _name: &str, _value: &str) -> ::std::result::Result<(), $crate::OptionViolation> {
                $( $crate::__params_set_options!(self, _name, _value;
                    $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
                ); )?

                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_set_tunables!(self, _name, _value;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $(
                    match self.$inner_field.set_option(_name, _value) {
                        Err($crate::OptionViolation::Unknown(_)) => {}
                        res => return res,
                    }
                )+

                Err($crate::OptionViolation::Unknown(_name.to_string()))
            }

            fn print_options(&self) {
                $( $crate::__params_print_options!(self;
                    $( [$option_key] $option : $option_ty => $option_default $(, $option_min, $option_max)?; )*
                ); )?

                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_print_tunable_options!(self;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $( self.$inner_field.print_options(); )+
            }

            fn print_tunables(&self) {
                #[cfg(not(feature = "release"))]
                {
                    $( $crate::__params_print_tunables!(@live self;
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }
                #[cfg(feature = "release")]
                {
                    $( $crate::__params_print_tunables!(@default
                        $( $tunable : $tunable_ty => $tunable_default, $tunable_min, $tunable_max, $tunable_c, $tunable_r; )*
                    ); )?
                }

                $( self.$inner_field.print_tunables(); )+
            }
        }
    };
}
