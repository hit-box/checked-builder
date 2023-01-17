use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, format_ident};
use syn::{DeriveInput, Error, Type};

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: &'a Ident,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Struct<'a> {
    pub fn new(
        ast: &'a DeriveInput,
        fields: impl Iterator<Item = &'a syn::Field>,
    ) -> Result<Struct<'a>, Error> {
        Ok(Struct {
            name: &ast.ident,
            fields: fields.map(Field::new).collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn builder_states(&self) -> Result<TokenStream, Error> {
        let state_trait_name = self.state_trait_name();
        let (states, (traits, impl_state)): (Vec<_>, (Vec<_>, Vec<_>)) = self
            .fields
            .iter()
            .map(|field| (field.builder_state(self), (field.field_setter_trait(self), field.impl_state_trait(self))))
            .unzip();
        let getter_traits = self
            .fields
            .iter()
            .map(|field| field.field_getter_trait(self));
        let impl_getters = self
            .fields
            .iter()
            .map(|field| field.impl_field_getters(self, self.fields.iter()));
        Ok(quote! {
            impl Config {
                pub fn builder() -> ConfigBuilderInitialState {
                    ConfigBuilderInitialState
                }
            }
            #[derive(Debug)] pub struct ConfigBuilderInitialState;
            impl ConfigBuilderState for ConfigBuilderInitialState {}
            pub trait #state_trait_name {}
            #( #states )*
            #( #traits )*
            #( #impl_state )*
            #( #getter_traits )*
            #( #impl_getters )*

            trait ConfigBuilder 
            where
                Self: ConfigBuilderState + ConfigBuilderEnableLogging + ConfigBuilderEnableTracing,
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
        })
    }

    pub fn state_trait_name(&self) -> Ident {
        let name = self.name;
        Ident::new(format!("{name}BuilderState").as_str(), Span::call_site())
    }
}

#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'a Ident,
    pub ty: &'a Type,
}

impl<'a> Field<'a> {
    pub fn new(field: &'a syn::Field) -> Result<Field<'a>, Error> {
        match field.ident {
            None => Err(Error::new_spanned(
                field,
                "CheckedBuilder unsupport nameless fields",
            )),
            Some(ref name) => Ok(Field {
                name,
                ty: &field.ty,
            }),
        }
    }

    pub fn builder_state_name(&self, structure: &Struct) -> Ident {
        let struct_name = structure.name;
        let state_name = format!(
            "{struct_name}Builder{}State",
            self.name.to_string().to_case(Case::Pascal)
        );
        Ident::new(state_name.as_str(), Span::call_site())
    }

    pub fn builder_state(&self, structure: &Struct) -> TokenStream {
        let state_trait_name = structure.state_trait_name();
        let Field { name, ty } = self;
        let state = self.builder_state_name(structure);
        quote! {
            #[derive(Debug)] pub struct #state<S: #state_trait_name> { inner: S, #name: #ty, }
        }
    }

    pub fn field_setter_trait(&self, structure: &Struct) -> TokenStream {
        let Field { name, ty } = self;
        let state_trait_name = structure.state_trait_name();
        let builder_state_name = self.builder_state_name(structure);
        let setter_trait_name = format!("{}Builder{}Set", structure.name, name.to_string().to_case(Case::Pascal));
        let setter_trait = Ident::new(setter_trait_name.as_str(), Span::call_site());
        quote! {
            trait #setter_trait: #state_trait_name + Sized {
                fn #name(self, value: #ty) -> #builder_state_name<Self>;
            }
            impl<S: #state_trait_name> #setter_trait for S {
                fn #name(self, value: #ty) -> #builder_state_name<S> {
                    #builder_state_name { inner: self, #name: value }
                }
            }
        }
    }

    pub fn impl_state_trait(&self, structure: &Struct) -> TokenStream {
        let builder_state_name = self.builder_state_name(structure);
        quote! {
            impl<S: ConfigBuilderState> ConfigBuilderState for #builder_state_name<S> {}
        }
    }

    pub fn field_getter_trait(&self, structure: &Struct) -> TokenStream {
        let Field { name, ty } = self;
        let builder_state_name = self.builder_state_name(structure);
        let getter_trait_name = format!("{}Builder{}", structure.name, name.to_string().to_case(Case::Pascal));
        let getter_trait = Ident::new(getter_trait_name.as_str(), Span::call_site());
        let getter_method_name = format_ident!("get_{}", name);
        quote! {
            trait #getter_trait { fn #getter_method_name(&self) -> #ty; }
            impl<S: ConfigBuilderState> #getter_trait for #builder_state_name<S> {
                fn #getter_method_name(&self) -> #ty { self.#name.clone() }
            }
        }
    }

    pub fn impl_field_getters(&self, structure: &Struct, fields: impl Iterator<Item = &'a Field<'a>>) -> TokenStream {
        let Field { name, ty } = self;
        let getter_method_name = format_ident!("get_{}", name);
        let getter_trait_name = format!("{}Builder{}", structure.name, name.to_string().to_case(Case::Pascal));
        let getter_trait = Ident::new(getter_trait_name.as_str(), Span::call_site());
        let impl_getters = fields
            .filter(|field| field.name != *name)
            .map(|field| {
                let builder_state_name = field.builder_state_name(structure);
                quote! {
                    impl<S> #getter_trait for #builder_state_name<S>
                    where
                        S: ConfigBuilderState + #getter_trait
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
