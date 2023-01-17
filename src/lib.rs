use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error, Data, Fields};

use crate::fields::Struct;

mod test;
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
                dbg!(&fields);
                let structure = Struct::new(&ast, fields.named.iter())?;
                dbg!(&structure);
                let builder_states = structure.builder_states()?;
                println!("{}", &builder_states.to_string());
                quote! {
                    #builder_states
                }
            },
            Fields::Unnamed(_) => return Err(Error::new_spanned(ast, "CheckedBuilder isn't support tuple structs")),
            Fields::Unit => return Err(Error::new_spanned(ast, "CheckedBuilder isn't support unit structs")),
        },
        Data::Enum(_) => return Err(Error::new_spanned(ast, "CheckedBuilder isn't support enums")),
        Data::Union(_) => return Err(Error::new_spanned(ast, "CheckedBuilder isn't support unions")),
    })
}
