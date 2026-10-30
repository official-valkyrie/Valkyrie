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

#[doc(hidden)]
#[macro_export]
macro_rules! __params_set_options {
    ($self_:ident, $name:ident, $value:ident;
     $( [$key:literal] $field:ident : $ty:ty => $default:expr $(, $min:expr, $max:expr)?; )*) => {
        $(
            if $name.eq_ignore_ascii_case($key) {
                return match $value.parse::<$ty>() {
                    Ok(new_value) => {
                        $(
                            let (min, max): ($ty, $ty) = ($min, $max);
                            if !(min..=max).contains(&new_value) {
                                return Err($crate::OptionViolation::OutOfRange {
                                    name: $name.to_string(),
                                    value: $value.to_string(),
                                    min: min.to_string(),
                                    max: max.to_string(),
                                });
                            }
                        )?
                        if $self_.$field == new_value {
                            return Err($crate::OptionViolation::Unchanged {
                                name: $name.to_string(),
                                value: $value.to_string(),
                            });
                        }
                        $self_.$field = new_value;
                        Ok(())
                    }
                    Err(_) if $value.is_empty() => Err($crate::OptionViolation::MissingValue {
                        name: $name.to_string(),
                    }),
                    Err(_) => Err($crate::OptionViolation::InvalidValue {
                        name: $name.to_string(),
                        value: $value.to_string(),
                    }),
                };
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_check_buttons {
    ($name:ident; $( $key:literal, )*) => {
        $(
            if $name.eq_ignore_ascii_case($key) {
                return Ok(());
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_set_tunables {
    ($self_:ident, $name:ident, $value:ident;
     $( $field:ident : $ty:ty => $default:expr, $min:expr, $max:expr, $c:expr, $r:expr; )*) => {
        $(
            if $name.eq_ignore_ascii_case(stringify!($field)) {
                return match $value.parse::<$ty>() {
                    Ok(new_value) => {
                        let (min, max): ($ty, $ty) = ($min, $max);
                        if !(min..=max).contains(&new_value) {
                            return Err($crate::OptionViolation::OutOfRange {
                                name: $name.to_string(),
                                value: $value.to_string(),
                                min: min.to_string(),
                                max: max.to_string(),
                            });
                        }
                        if $self_.$field == new_value {
                            return Err($crate::OptionViolation::Unchanged {
                                name: $name.to_string(),
                                value: $value.to_string(),
                            });
                        }
                        $self_.$field = new_value;
                        Ok(())
                    }
                    Err(_) if $value.is_empty() => Err($crate::OptionViolation::MissingValue {
                        name: $name.to_string(),
                    }),
                    Err(_) => Err($crate::OptionViolation::InvalidValue {
                        name: $name.to_string(),
                        value: $value.to_string(),
                    }),
                };
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_set_variables {
    ($self_:ident, $name:ident, $value:ident;
     $( $field:ident : $ty:ty = $default:expr; )*) => {
        $(
            if $name.eq_ignore_ascii_case(stringify!($field)) {
                return match $value.parse::<$ty>() {
                    Ok(new_value) => {
                        $self_.$field = new_value;
                        Ok(())
                    }
                    Err(_) if $value.is_empty() => Err($crate::OptionViolation::MissingValue {
                        name: $name.to_string(),
                    }),
                    Err(_) => Err($crate::OptionViolation::InvalidValue {
                        name: $name.to_string(),
                        value: $value.to_string(),
                    }),
                };
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_visit_options {
    ($self_:ident, $visit:ident;
     $( [$key:literal] $field:ident : $ty:ty => $default:expr $(, $min:expr, $max:expr)?; )*) => {
        $(
            {
                #[allow(unused_variables)]
                let range: ::std::option::Option<(::std::string::String, ::std::string::String)> = None;
                $( let range = Some(($min.to_string(), $max.to_string())); )?
                $visit(&$crate::OptionSpec {
                    name: $key.to_string(),
                    kind: <$ty as $crate::UciOptionType>::KIND,
                    default: $self_.$field.to_string(),
                    range,
                    tunable: false,
                });
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_visit_buttons {
    ($visit:ident; $( $key:literal, )*) => {
        $(
            $visit(&$crate::OptionSpec {
                name: $key.to_string(),
                kind: $crate::OptionKind::Button,
                default: ::std::string::String::new(),
                range: None,
                tunable: false,
            });
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_visit_tunable_options {
    ($self_:ident, $visit:ident;
     $( $field:ident : $ty:ty => $default:expr, $min:expr, $max:expr, $c:expr, $r:expr; )*) => {
        $(
            $visit(&$crate::OptionSpec {
                name: stringify!($field).to_string(),
                kind: <$ty as $crate::UciOptionType>::KIND,
                default: $self_.$field.to_string(),
                range: Some(($min.to_string(), $max.to_string())),
                tunable: true,
            });
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_print_tunables {
    (@live $self_:ident, $print:ident;
     $( $field:ident : $ty:ty => $default:expr, $min:expr, $max:expr, $c:expr, $r:expr; )*) => {
        $(
            $print(&format!(
                "{}, {}, {}, {}, {}, {}, {}",
                stringify!($field),
                <$ty as $crate::UciOptionType>::TUNE_KIND,
                $self_.$field,
                $min,
                $max,
                $c,
                $r
            ));
        )*
    };
    (@default $print:ident;
     $( $field:ident : $ty:ty => $default:expr, $min:expr, $max:expr, $c:expr, $r:expr; )*) => {
        $(
            $print(&format!(
                "{}, {}, {}, {}, {}, {}, {}",
                stringify!($field),
                <$ty as $crate::UciOptionType>::TUNE_KIND,
                $default,
                $min,
                $max,
                $c,
                $r
            ));
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_option_accessors {
    ($( [$key:literal] $field:ident : $ty:ty => $default:expr $(, $min:expr, $max:expr)?; )*) => {
        $(
            pub fn $field(&self) -> $ty {
                self.$field.clone()
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_tunable_accessors {
    ($( $(#[$meta:meta])* $field:ident : $ty:ty => $default:expr, $min:expr, $max:expr, $c:expr, $r:expr; )*) => {
        $(
            $(#[$meta])*
            #[cfg(not(feature = "release"))]
            pub const fn $field(&self) -> $ty {
                self.$field
            }

            $(#[$meta])*
            #[cfg(feature = "release")]
            #[inline(always)]
            pub const fn $field(&self) -> $ty {
                $default
            }
        )*
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __params_variable_accessors {
    ($( $(#[$meta:meta])* $field:ident : $ty:ty = $default:expr; )*) => {
        $(
            $(#[$meta])*
            pub const fn $field(&self) -> $ty {
                self.$field
            }

            $crate::paste::paste! {
                pub fn [< set_ $field >](&mut self, value: $ty) {
                    self.$field = value;
                }
            }
        )*
    };
}
