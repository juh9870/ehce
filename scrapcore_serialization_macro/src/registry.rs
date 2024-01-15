use crate::registry::parser::parse_struct_defs;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use rustc_hash::FxHashMap;
use std::ops::Deref;
use syn::{parse_macro_input, ItemStruct, Type};

use crate::SERIALIZATION_CRATE;

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
    /// Model name in snake_case, for usage as field name
    field_name: Ident,
    /// Model name in PascalCase, for usage as enum variant name
    variant_name: Ident,
    ty: Type,
    ty_serialized: Type,
}

#[derive(Debug, Default)]
struct ModelSet(Vec<ModelKind>);

impl ModelSet {
    fn variants(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.iter().map(
            |ModelKind {
                 variant_name, ty, ..
             }| { quote!(#variant_name(#ty)) },
        )
    }
    fn serialized_variants(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.iter().map(
            |ModelKind {
                 variant_name,
                 ty_serialized,
                 ..
             }| { quote!(#variant_name(#ty_serialized),) },
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
    serialized_model_name: Ident,
    schema: bool,

    singletons: ModelSet,
    registries: ModelSet,
    assets: FxHashMap<Ident, Type>,
}

fn registry_impl_inner(
    attr: proc_macro::TokenStream,
    mut item_struct: ItemStruct,
) -> syn::Result<TokenStream> {
    let definitions = parse_struct_defs(attr, &mut item_struct)?;

    let model = definitions.model();

    Ok(quote! {
        #model
    })
}

impl RegistryDefinitions {
    fn model(&self) -> TokenStream {
        let singletons = self.singletons.serialized_variants();
        let registries = self.registries.serialized_variants();
        let serialized_model_name = &self.serialized_model_name;
        let schema_derive = if self.schema {
            quote!(#[derive(schemars::JsonSchema)])
        } else {
            quote!()
        };
        let model_enum = quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            #schema_derive
            #[serde(tag = "type")]
            #[serde(rename_all = "PascalCase")]
            #[serde(rename = "DatabaseItem")]
            enum #serialized_model_name {
                #(#singletons)*
                #(#registries)*
            }
        };

        model_enum
    }
}

fn serialized_of(ty: &Type) -> syn::Result<Type> {
    let ser = SERIALIZATION_CRATE.deref();
    syn::parse2(quote! {
        <#ty as #ser::serialization::SerializationFallback>::Fallback
    })
}
