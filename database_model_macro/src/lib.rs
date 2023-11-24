use proc_macro::TokenStream;

use attribute_derive::Attribute;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Error, GenericArgument, Item, Meta, PathArguments, Type};

use crate::enums::process_enum;
use crate::structs::process_struct;

mod enums;
mod structs;

fn serialization_mod() -> proc_macro2::TokenStream {
    quote!(crate::model::serialization)
}

fn model_mod() -> proc_macro2::TokenStream {
    quote!(crate::model)
}

#[derive(Debug, Attribute)]
struct AttributeInput {
    name: Option<String>,
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
        if &name.ident.to_string() != "model_attr" {
            continue;
        }

        let tokens = &list.tokens;
        inner_attrs.push(quote_spanned!(attr.span()=>#[#tokens]));
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
        _ => Err(Error::new(Span::call_site(), "")),
    } {
        Ok(data) => data,
        Err(err) => err.to_compile_error().into(),
    }
}
