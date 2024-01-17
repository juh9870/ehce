use proc_macro::TokenStream;

use attribute_derive::Attribute;
use lazy_static::lazy_static;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_str, Error, Item, Meta, Type};

use crate::enums::process_enum;
use crate::registry::registry_impl;
use crate::structs::process_struct;

mod enums;
mod structs;

mod registry;

fn serialization_mod() -> proc_macro2::TokenStream {
    quote!(crate::model::serialization)
}

fn model_mod() -> proc_macro2::TokenStream {
    quote!(crate::model)
}

#[derive(Debug)]
struct IdentSync(String);

impl IdentSync {
    fn join(&self, path: &str) -> IdentSync {
        IdentSync(format!("{}::{}", self.0, path))
    }
}

impl ToTokens for IdentSync {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty: syn::Type = parse_str(&self.0).unwrap();
        // proc_macro2::Ident::new(&self.0, Span::call_site()).to_tokens(tokens)
        ty.to_tokens(tokens)
    }
}

lazy_static! {
    static ref SERIALIZATION_CRATE: IdentSync = crate_name("scrapcore_serialization");
    static ref MOD_REGISTRY: IdentSync = SERIALIZATION_CRATE.join("registry");
    static ref MOD_SERIALIZATION: IdentSync = SERIALIZATION_CRATE.join("serialization");
}
fn crate_name(name: &str) -> IdentSync {
    match proc_macro_crate::crate_name(name) {
        Ok(data) => IdentSync(match data {
            proc_macro_crate::FoundCrate::Itself => "crate".to_string(),
            proc_macro_crate::FoundCrate::Name(name) => name,
        }),
        Err(_) => panic!("Crate `{name}` is not found in scope."),
    }
}

#[derive(Debug, Attribute)]
struct AttributeInput {
    name: Option<String>,
    no_schema: bool,
}

impl AttributeInput {
    fn schema_derive(&self) -> Option<proc_macro2::TokenStream> {
        if self.no_schema {
            None
        } else {
            Some(quote! {
                #[derive(schemars::JsonSchema)]
            })
        }
    }
}

fn fallthrough(attrs: &mut Vec<syn::Attribute>) -> Vec<proc_macro2::TokenStream> {
    let mut inner_attrs = vec![];
    let mut i = 0;
    while i < attrs.len() {
        let attr = &attrs[i];
        i += 1;
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let Some(name) = list.path.segments.last() else {
            continue;
        };
        let tokens = &list.tokens;
        match name.ident.to_string().as_str() {
            "model_attr" => {
                inner_attrs.push(quote_spanned!(attr.span()=>#[#tokens]));
            }
            "model_serde" => {
                inner_attrs.push(quote_spanned!(attr.span()=>#[serde(#tokens)]));
            }
            _ => continue,
        }

        i -= 1;
        attrs.remove(i);
    }
    inner_attrs
}

fn serialized_type(ty: &Type) -> Result<proc_macro2::TokenStream, Error> {
    let serialization_mod = serialization_mod();
    Ok(quote_spanned! {ty.span()=>
        <#ty as #serialization_mod::ModelDeserializableFallbackType>::Serialized
    })
}

#[proc_macro_attribute]
pub fn database_model(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data: Item = parse_macro_input!(input);

    match match data {
        Item::Struct(data) => process_struct(attr, data),
        Item::Enum(data) => process_enum(attr, data),
        _ => Err(Error::new(Span::call_site(), "Invalid input")),
    } {
        Ok(data) => data,
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn registry(attr: TokenStream, input: TokenStream) -> TokenStream {
    registry_impl(attr, input)
}
