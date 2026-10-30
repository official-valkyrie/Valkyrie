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
macro_rules! define_engine_config {
    (
        general: $general_ty:ty,
        strategies {
            $( $s_assoc:ident : $s_bound:ident | $s_method:ident | $s_default:ty ),+ $(,)?
        }
    ) => {
        pub trait EngineConfig: Send + Sync + Sized + 'static {
            $( type $s_assoc: Strategy; )+
            type Edge: $crate::engine::tree::components::EdgeType;
            type Node: $crate::engine::tree::components::NodeType<Edge = Self::Edge>;
        }

        pub struct EngineBuilder<
            $( $s_assoc = $s_default, )+
            NodeSlot = Unspecified,
        >(
            ::std::marker::PhantomData<fn() -> ($( $s_assoc, )+ NodeSlot)>,
        );

        impl EngineBuilder {
            pub fn new() -> Self {
                EngineBuilder(::std::marker::PhantomData)
            }
        }

        impl Default for EngineBuilder {
            fn default() -> Self {
                Self::new()
            }
        }

        $crate::define_engine_config!(@setters
            []
            [ $( $s_assoc | $s_method ),+ ]
        );

        impl< $( $s_assoc, )+ NodeSlot > EngineBuilder< $( $s_assoc, )+ NodeSlot > {
            pub fn node<__N>(self) -> EngineBuilder< $( $s_assoc, )+ __N > {
                EngineBuilder(::std::marker::PhantomData)
            }
        }

        pub struct GenericConfig< $( $s_assoc, )+ NodeSlot >(
            ::std::marker::PhantomData<fn() -> ($( $s_assoc, )+ NodeSlot)>,
        );

        impl< $( $s_assoc, )+ NodeSlot > EngineConfig for GenericConfig< $( $s_assoc, )+ NodeSlot >
        where
            $( $s_assoc: Strategy, )+
            NodeSlot: $crate::engine::tree::components::NodeType,
        {
            $( type $s_assoc = $s_assoc; )+
            type Node = NodeSlot;
            type Edge = <NodeSlot as $crate::engine::tree::components::NodeType>::Edge;
        }

        #[doc(hidden)]
        pub trait Buildable: EngineConfig {}

        impl< $( $s_assoc, )+ NodeSlot > Buildable for GenericConfig< $( $s_assoc, )+ NodeSlot >
        where
            Self: EngineConfig,
            $( $s_assoc: $s_bound<Self>, )+
        {
        }

        impl< $( $s_assoc, )+ NodeSlot > EngineBuilder< $( $s_assoc, )+ NodeSlot >
        where
            GenericConfig< $( $s_assoc, )+ NodeSlot >: Buildable,
        {
            pub fn build(self) -> $crate::Engine<GenericConfig< $( $s_assoc, )+ NodeSlot >> {
                $crate::Engine::new(EngineParams::new())
            }
        }

        #[derive(Debug)]
        pub struct EngineParams<C: EngineConfig> {
            general: $general_ty,
            $( $s_method: <C::$s_assoc as Strategy>::Params, )+
        }

        impl<C: EngineConfig> EngineParams<C> {
            const OPTION_NAMES: $crate::OptionNames = $crate::OptionNames {
                own: &[],
                inner: &[
                    &<$general_ty as $crate::StrategyParams>::OPTION_NAMES,
                    $( &<<C::$s_assoc as Strategy>::Params as $crate::StrategyParams>::OPTION_NAMES, )+
                ],
            };

            pub fn new() -> Self {
                const { Self::OPTION_NAMES.assert_unique() }

                Self {
                    general: <$general_ty as $crate::StrategyParams>::new(),
                    $( $s_method: <<C::$s_assoc as Strategy>::Params as $crate::StrategyParams>::new(), )+
                }
            }

            pub fn general(&self) -> &$general_ty {
                &self.general
            }

            $(
                pub fn $s_method(&self) -> &<C::$s_assoc as Strategy>::Params {
                    &self.$s_method
                }
            )+

            pub fn set_option(&mut self, name: &str, value: &str) -> ::std::result::Result<(), $crate::OptionViolation> {
                match $crate::StrategyParams::set_option(&mut self.general, name, value) {
                    Err($crate::OptionViolation::Unknown(_)) => {}
                    res => return res,
                }
                $(
                    match $crate::StrategyParams::set_option(&mut self.$s_method, name, value) {
                        Err($crate::OptionViolation::Unknown(_)) => {}
                        res => return res,
                    }
                )+
                Err($crate::OptionViolation::Unknown(name.to_string()))
            }

            pub fn visit_options(&self, visit: &dyn Fn(&$crate::OptionSpec)) {
                $crate::StrategyParams::visit_options(&self.general, visit);
                $( $crate::StrategyParams::visit_options(&self.$s_method, visit); )+
                $crate::StrategyParams::visit_tunable_options(&self.general, visit);
                $( $crate::StrategyParams::visit_tunable_options(&self.$s_method, visit); )+
            }

            pub fn print_tunables(&self, print: &dyn Fn(&str)) {
                $crate::StrategyParams::print_tunables(&self.general, print);
                $( $crate::StrategyParams::print_tunables(&self.$s_method, print); )+
            }
        }

        impl<C: EngineConfig> Default for EngineParams<C> {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    ( @setters
        [ $( $b_assoc:ident | $b_method:ident, )* ]
        [ $cur_assoc:ident | $cur_method:ident
          $( , $r_assoc:ident | $r_method:ident )* ]
    ) => {
        impl< $( $b_assoc, )* $cur_assoc, $( $r_assoc, )* NodeSlot >
            EngineBuilder< $( $b_assoc, )* $cur_assoc, $( $r_assoc, )* NodeSlot >
        {
            pub fn $cur_method<__S>(self)
                -> EngineBuilder< $( $b_assoc, )* __S, $( $r_assoc, )* NodeSlot >
            {
                EngineBuilder(::std::marker::PhantomData)
            }
        }

        $crate::define_engine_config!(@setters
            [ $( $b_assoc | $b_method, )* $cur_assoc | $cur_method, ]
            [ $( $r_assoc | $r_method ),* ]
        );
    };

    ( @setters [ $( $b_assoc:ident | $b_method:ident, )* ] [] ) => {};
}
