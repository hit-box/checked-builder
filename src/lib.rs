use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

use crate::fields::Struct;

mod fields;
mod test;

#[proc_macro_derive(CheckedBuilder, attributes(builder))]
pub fn derive_checked_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match checked_builder_derive_imp(&input) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn checked_builder_derive_imp(ast: &syn::DeriveInput) -> Result<TokenStream, Error> {
    Ok(match &ast.data {
        Data::Struct(structure) => match &structure.fields {
            Fields::Named(fields) => {
                let structure = Struct::new(&ast, fields.named.iter())?;
                let struct_builder_impl = structure.struct_builder_impl();
                let state_trait_definition = structure.state_trait.definition();
                let initial_state_definition = structure.initial_state.definition();
                let initial_state_trait_impl = structure.initial_state_trait_impl();
                let builder_states = structure.builder_states();
                let setter_traits = structure.setter_traits();
                let getter_traits = structure.getter_traits();
                let getters_impl = structure.getters_impl();

                quote! {
                    #struct_builder_impl
                    #state_trait_definition
                    #initial_state_definition
                    #initial_state_trait_impl
                    mod config_builder {
                        use super::*;
                        #builder_states
                        #setter_traits
                        #getter_traits
                        #getters_impl

                        pub trait ConfigBuilder
                        where
                            Self: ConfigBuilderState + ConfigBuilderEnableLogging + ConfigBuilderEnableTracing + ConfigBuilderHost,
                        {
                            fn build(self) -> Config;
                        }

                        impl<S> ConfigBuilder for S
                        where
                            S: ConfigBuilderState + ConfigBuilderEnableLogging + ConfigBuilderEnableTracing + ConfigBuilderHost,
                        {
                            fn build(self) -> Config {
                                Config {
                                    enable_logging: self.get_enable_logging(),
                                    enable_tracing: self.get_enable_tracing(),
                                    host: self.get_host(),
                                }
                            }
                        }
                    }
                    use config_builder::{ConfigBuilder, ConfigBuilderEnableLoggingSetter, ConfigBuilderEnableTracingSetter, ConfigBuilderHostSetter};
                }
            }
            Fields::Unnamed(_) => {
                return Err(Error::new_spanned(
                    ast,
                    "CheckedBuilder isn't support tuple structs",
                ))
            }
            Fields::Unit => {
                return Err(Error::new_spanned(
                    ast,
                    "CheckedBuilder isn't support unit structs",
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new_spanned(
                ast,
                "CheckedBuilder isn't support enums",
            ))
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                ast,
                "CheckedBuilder isn't support unions",
            ))
        }
    })
}
