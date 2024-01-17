use std::ops::Deref;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use rustc_hash::FxHashMap;
use syn::{parse_macro_input, ItemStruct, Type};

use crate::registry::parser::parse_struct_defs;
use crate::{MOD_REGISTRY, MOD_SERIALIZATION};

mod parser;

pub fn registry_impl(
    attr: proc_macro::TokenStream,
    item_struct: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = parse_macro_input!(item_struct);
    match registry_impl_inner(attr, item_struct) {
        Ok(data) => {
            #[cfg(feature = "debug_output")]
            match syn::parse(data.clone().into()) {
                Ok(file) => {
                    eprintln!("=============================");
                    eprintln!("Source:\n{}", prettyplease::unparse(&file));
                    eprintln!("=============================");
                }
                Err(err) => {
                    eprintln!("=============================");
                    eprintln!("Code parsing error:\n{}", err);
                    eprintln!("=============================");
                    eprintln!("Source:\n{}", data);
                    eprintln!("=============================");
                }
            }
            data.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

#[derive(Debug)]
struct ModelKind {
    span: Span,
    /// Model name in snake_case, for usage as field name
    field_name: Ident,
    /// Model name in PascalCase, for usage as enum variant name
    variant_name: Ident,
    /// Raw model name in snake_case, for usage in raw field names
    raw_field_name: Ident,
    ty: Type,
    ty_serialized: Type,
}

#[derive(Debug, Default)]
struct ModelSet(Vec<ModelKind>);

impl ModelSet {
    fn variants(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.iter().map(
            |ModelKind {
                 variant_name,
                 ty,
                 span,
                 ..
             }| { quote_spanned!(*span=>#variant_name(#ty)) },
        )
    }
    fn serialized_variants(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.iter().map(
            |ModelKind {
                 variant_name,
                 ty_serialized,
                 span,
                 ..
             }| { quote_spanned!(*span=>#variant_name(#ty_serialized),) },
        )
    }
}

impl Deref for ModelSet {
    type Target = Vec<ModelKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct RegistryDefinitions {
    pascal_name: Ident,
    model_name: Ident,
    serialized_model_name: Ident,
    registry_name: Ident,
    partial_registry_name: Ident,
    schema: bool,

    singletons: ModelSet,
    collections: ModelSet,
    assets: FxHashMap<Ident, Type>,
}

fn registry_impl_inner(
    attr: proc_macro::TokenStream,
    mut item_struct: ItemStruct,
) -> syn::Result<TokenStream> {
    let definitions = parse_struct_defs(attr, &mut item_struct)?;

    let model = definitions.model();
    let partial_registry = definitions.partial_registry();

    Ok(quote! {
        #model
        #partial_registry
    })
}

impl RegistryDefinitions {
    fn model(&self) -> TokenStream {
        let singletons = self.singletons.serialized_variants();
        let registries = self.collections.serialized_variants();
        let serialized_model_name = &self.serialized_model_name;
        let schema_derive = if self.schema {
            quote!(#[derive(schemars::JsonSchema)])
        } else {
            quote!()
        };
        let model_name_str = self.model_name.to_string();
        let model_enum = quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            #schema_derive
            #[serde(tag = "type")]
            #[serde(rename_all = "PascalCase")]
            #[serde(rename = #model_name_str)]
            enum #serialized_model_name {
                #(#singletons)*
                #(#registries)*
            }
        };

        model_enum
    }

    fn partial_registry(&self) -> TokenStream {
        let Self {
            partial_registry_name,
            singletons,
            collections,
            ..
        } = self;
        let reg = MOD_REGISTRY.deref();
        let singletons = singletons.iter().map(
            |ModelKind {
                 span,
                 field_name,
                 ty,
                 ..
             }| {
                quote_spanned! {*span=>
                    #field_name: #reg::PartialSingleton<#ty>,
                }
            },
        );
        let collections = collections.iter().map(
            |ModelKind {
                 span,
                 field_name,
                 raw_field_name,
                 ty,
                 ..
             }| {
                quote_spanned! {*span=>
                    #field_name: #reg::PartialItemCollection<#ty>,
                    #raw_field_name: #reg::RawItemCollection<#ty>,
                }
            },
        );
        quote! {
            #[derive(Debug, Default)]
            struct #partial_registry_name {
                #(#singletons)*
                #(#collections)*
            }
        }
    }
}

fn serialized_of(ty: &Type) -> syn::Result<Type> {
    let ser = MOD_SERIALIZATION.deref();
    syn::parse2(quote! {
        <#ty as #ser::SerializationFallback>::Fallback
    })
}
