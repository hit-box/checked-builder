use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, Error, Token, Type, VisPublic, Visibility};

const STATE_TRAIT_NAME_POSTFIX: &str = "BuilderState";
const INITIAL_STATE_NAME_POSTFIX: &str = "BuilderInitialState";
const SETTER_TRAIT_POSTFIX: &str = "Setter";

#[derive(Debug)]
pub struct StateTrait<'a> {
    pub vis: &'a Visibility,
    pub name: Ident,
}

impl<'a> StateTrait<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        let struct_name = &ast.ident;
        let vis = &ast.vis;
        Self {
            name: Ident::new(
                format!("{struct_name}{STATE_TRAIT_NAME_POSTFIX}").as_str(),
                Span::call_site(),
            ),
            vis,
        }
    }

    pub(crate) fn definition(&self) -> TokenStream {
        let StateTrait { vis, name } = self;
        quote! {
            #vis trait #name {}
        }
    }
}

#[derive(Debug)]
pub struct InitialState<'a> {
    pub vis: &'a Visibility,
    pub name: Ident,
}

impl<'a> InitialState<'a> {
    pub fn new(ast: &'a DeriveInput) -> Self {
        let struct_name = &ast.ident;
        Self {
            vis: &ast.vis,
            name: Ident::new(
                format!("{struct_name}{INITIAL_STATE_NAME_POSTFIX}").as_str(),
                Span::call_site(),
            ),
        }
    }

    pub fn definition(&self) -> TokenStream {
        let InitialState { vis, name } = self;
        quote! {
            #[derive(Debug)] #vis struct #name;
        }
    }
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: &'a Ident,
    pub state_trait: StateTrait<'a>,
    pub initial_state: InitialState<'a>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Struct<'a> {
    pub fn new(
        ast: &'a DeriveInput,
        fields: impl Iterator<Item = &'a syn::Field>,
    ) -> Result<Struct<'a>, Error> {
        let name = &ast.ident;
        let state_trait = StateTrait::new(ast);
        let initial_state = InitialState::new(ast);
        let fields = fields
            .map(|field| Field::new(field, name))
            .collect::<Result<_, _>>()?;
        Ok(Struct {
            name,
            state_trait,
            initial_state,
            fields,
        })
    }

    pub(crate) fn struct_builder_impl(&self) -> TokenStream {
        let struct_name = self.name;
        let initial_state_name = &self.initial_state.name;
        quote! {
            impl #struct_name {
                pub fn builder() -> #initial_state_name {
                    #initial_state_name
                }
            }
        }
    }

    pub(crate) fn initial_state_trait_impl(&self) -> TokenStream {
        let state_trait_name = &self.state_trait.name;
        let initial_state_name = &self.initial_state.name;
        quote! {
            impl #state_trait_name for #initial_state_name {}
        }
    }

    pub fn builder_states(&self) -> TokenStream {
        let states = self.fields.iter().map(|field| field.builder_state(self));
        quote! {
            #( #states )*
        }
    }

    pub fn setter_traits(&self) -> TokenStream {
        let setters = self.fields.iter().map(|field| field.setter_trait(self));
        quote! {
            #( #setters )*
        }
    }

    pub fn getter_traits(&self) -> TokenStream {
        let getters = self.fields.iter().map(|field| field.getter_trait(self));
        quote! {
            #( #getters )*
        }
    }

    pub(crate) fn getters_impl(&self) -> TokenStream {
        let getters_impl = self.fields.iter().map(|field| field.getters_impl(self));
        quote! {
            #( #getters_impl )*
        }
    }
}

#[derive(Debug)]
pub struct BuilderState {
    pub vis: Visibility,
    pub name: Ident,
}

impl BuilderState {
    pub fn new(field_name: &Ident, struct_name: &Ident) -> Self {
        let name = Ident::new(
            format!(
                "{struct_name}Builder{}State",
                field_name.to_string().to_case(Case::Pascal)
            )
            .as_str(),
            Span::call_site(),
        );
        Self {
            name,
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](Span::call_site()),
            }),
        }
    }
}

#[derive(Debug)]
pub struct SetterTrait {
    pub name: Ident,
}

impl SetterTrait {
    pub fn new(field_name: &Ident, struct_name: &Ident) -> Self {
        let name = field_name.to_string().to_case(Case::Pascal);
        let setter_trait_name = format!("{struct_name}Builder{name}{SETTER_TRAIT_POSTFIX}");
        Self {
            name: Ident::new(setter_trait_name.as_str(), Span::call_site()),
        }
    }
}

#[derive(Debug)]
pub struct GetterTrait {
    pub name: Ident,
    pub method_name: Ident,
}

impl GetterTrait {
    pub fn new(field_name: &Ident, struct_name: &Ident) -> Self {
        let name = field_name.to_string().to_case(Case::Pascal);
        let getter_trait_name = format!("{struct_name}Builder{name}");
        let name = Ident::new(getter_trait_name.as_str(), Span::call_site());
        Self {
            name,
            method_name: format_ident!("get_{field_name}"),
        }
    }
}

#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'a Ident,
    pub builder_state: BuilderState,
    pub setter_trait: SetterTrait,
    pub getter_trait: GetterTrait,
    pub ty: &'a Type,
}

impl<'a> Field<'a> {
    pub fn new(field: &'a syn::Field, struct_name: &'a Ident) -> Result<Field<'a>, Error> {
        match field.ident {
            None => Err(Error::new_spanned(
                field,
                "CheckedBuilder don't support nameless fields",
            )),
            Some(ref name) => {
                let builder_state = BuilderState::new(name, struct_name);
                let setter_trait = SetterTrait::new(name, struct_name);
                let getter_trait = GetterTrait::new(name, struct_name);
                Ok(Field {
                    name,
                    builder_state,
                    setter_trait,
                    getter_trait,
                    ty: &field.ty,
                })
            }
        }
    }

    pub fn builder_state(&self, structure: &Struct) -> TokenStream {
        let state_trait_name = &structure.state_trait.name;
        let Field { name, ty, .. } = self;
        let state = &self.builder_state.name;
        quote! {
            #[derive(Debug)] pub struct #state<S: #state_trait_name> { inner: S, #name: #ty, }
            impl<S: #state_trait_name> #state_trait_name for #state<S> {}
        }
    }

    pub fn setter_trait(&self, structure: &Struct) -> TokenStream {
        let Field { name, ty, .. } = self;
        let state_trait_name = &structure.state_trait.name;
        let builder_state_name = &self.builder_state.name;
        let setter_trait_name = &self.setter_trait.name;
        quote! {
            pub trait #setter_trait_name: #state_trait_name + Sized {
                fn #name(self, value: #ty) -> #builder_state_name<Self>;
            }
            impl<S: #state_trait_name> #setter_trait_name for S {
                fn #name(self, value: #ty) -> #builder_state_name<S> {
                    #builder_state_name { inner: self, #name: value }
                }
            }
        }
    }

    pub fn getter_trait(&self, structure: &Struct) -> TokenStream {
        let Field { name, ty, .. } = self;
        let builder_state_name = &self.builder_state.name;
        let state_trait_name = &structure.state_trait.name;
        let getter_trait_name = &self.getter_trait.name;
        let getter_method_name = &self.getter_trait.method_name;
        quote! {
            pub trait #getter_trait_name { fn #getter_method_name(&self) -> #ty; }
            impl<S: #state_trait_name> #getter_trait_name for #builder_state_name<S> {
                fn #getter_method_name(&self) -> #ty { self.#name.clone() }
            }
        }
    }

    pub fn getters_impl(
        &self,
        structure: &Struct,
        // fields: impl Iterator<Item = &'a Field<'a>>,
    ) -> TokenStream {
        let Field { name, ty, .. } = self;
        let getter_method_name = &self.getter_trait.method_name;
        let getter_trait_name = &self.getter_trait.name;
        let state_trait_name = &structure.state_trait.name;
        let impl_getters = structure
            .fields
            .iter()
            .filter(|field| field.name != *name)
            .map(|field| {
                let builder_state_name = &field.builder_state.name;
                quote! {
                    impl<S> #getter_trait_name for #builder_state_name<S>
                    where
                        S: #state_trait_name + #getter_trait_name,
                    {
                        fn #getter_method_name(&self) -> #ty { self.inner.#getter_method_name() }
                    }
                }
            });
        quote! {
            #( #impl_getters )*
        }
    }
}
