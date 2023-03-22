use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

use crate::fields::Struct;

mod fields;

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
                let builder_trait = structure.builder_trait();
                let builder_trait_impl = structure.builder_trait_impl();
                let import_traits = structure.import_traits();
                let module_name = structure.module_name();

                quote! {
                    #struct_builder_impl
                    #state_trait_definition
                    #initial_state_definition
                    #initial_state_trait_impl
                    mod #module_name {
                        use super::*;
                        #builder_states
                        #setter_traits
                        #getter_traits
                        #getters_impl
                        #builder_trait
                        #builder_trait_impl
                    }
                    #import_traits
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
