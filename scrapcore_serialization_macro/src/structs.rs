use proc_macro::TokenStream;
use std::hash::{BuildHasher, BuildHasherDefault};

use attribute_derive::Attribute;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote, quote_spanned};
use rustc_hash::FxHasher;
use syn::spanned::Spanned;
use syn::{Error, ItemStruct, Type};

use crate::{fallthrough, model_mod, serialization_mod, serialized_type, AttributeInput};

#[derive(Debug)]
enum Modifier {
    Min(Literal),
    Max(Literal),
}

#[derive(Debug)]
struct FieldData {
    name: Ident,
    original_type: Type,
    definition: proc_macro2::TokenStream,
    modifiers: Vec<Modifier>,
}

#[derive(Debug, Attribute)]
#[attribute(ident = model)]
struct FieldAttributeInput {
    /// Applies min validator to the field
    min: Option<Literal>,
    /// Applies max validator to the field
    max: Option<Literal>,
    /// Generated AsRef implementation for marked struct to value of this field
    as_ref: bool,
    /// Custom serialized field type
    ty: Option<Type>,
}

impl FieldAttributeInput {
    fn apply(self, data: &mut FieldData) {
        if let Some(min) = self.min {
            data.modifiers.push(Modifier::Min(min));
        }
        if let Some(max) = self.max {
            data.modifiers.push(Modifier::Max(max));
        }
    }
}

pub fn process_struct(attr: TokenStream, mut data: ItemStruct) -> Result<TokenStream, Error> {
    let mut fields = Vec::new();
    let model_fallthrough_attrs = fallthrough(&mut data.attrs);

    let serialization_mod = serialization_mod();
    let model_mod = model_mod();

    let attr = AttributeInput::from_args(attr.into())?;

    let model_name = &data.ident;
    let serialized_name = attr
        .name
        .as_ref()
        .map(|e| format_ident!("{e}"))
        .unwrap_or_else(|| format_ident!("{}Serialized", data.ident));

    let mut as_refs = vec![];

    for field in &mut data.fields {
        let Some(name) = &field.ident else {
            return Err(Error::new(field.span(), "All model fields must be named"));
        };
        let ty = &field.ty;

        let attribute_data = FieldAttributeInput::remove_attributes(&mut field.attrs)?;

        if attribute_data.as_ref {
            as_refs.push(quote! {
                #[automatically_derived]
                impl AsRef<#ty> for #model_name {
                    fn as_ref(&self) -> &#ty {
                        &self.#name
                    }
                }
            })
        }

        let serialized_type = if let Some(ty) = &attribute_data.ty {
            quote!(#ty)
        } else {
            serialized_type(ty)?
        };
        let fallthrough_attrs = fallthrough(&mut field.attrs);
        let definition = quote_spanned!(field.span()=>
            #(#fallthrough_attrs)*
            #name: #serialized_type
        );

        let mut field_data = FieldData {
            name: name.clone(),
            modifiers: Vec::new(),
            definition,
            original_type: ty.clone(),
        };
        attribute_data.apply(&mut field_data);

        fields.push(field_data)
    }

    let tokens = fields.iter().map(|e| &e.definition);
    let schema_derive = attr.schema_derive();
    let model_name_str = model_name.to_string();
    let serialized_struct = quote!(
        #(#model_fallthrough_attrs)*
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename = #model_name_str)]
        #schema_derive
        #[serde(rename_all = "camelCase")]
        pub struct #serialized_name {
            #(#tokens),*
        }

        #[automatically_derived]
        impl #serialization_mod::ModelDeserializableFallbackType for #model_name {
            type Serialized = #serialized_name;
        }

        #(#as_refs)*

        #[automatically_derived]
        impl AsRef<#model_name> for #model_name {
            fn as_ref(&self) -> &#model_name {
                &self
            }
        }
    );

    let map_name = attr.name.unwrap_or_else(|| {
        model_name
            .to_string()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
    });
    let kind_name = map_name.from_case(Case::Snake).to_case(Case::Pascal);
    let _map_name = format_ident!("{}", map_name);
    let _kind_name = format_ident!("{}", kind_name);

    let names = fields.iter().map(|e| &e.name);
    let hasher = BuildHasherDefault::<FxHasher>::default();

    let _reservation_field_name = format_ident!("reserved_{}__", hasher.hash_one(&data.ident));
    let serialized_field_name = format_ident!("serialized_{}__", hasher.hash_one(&data.ident));

    let modifiers = fields.iter().map(|f| {
        let name = &f.name;
        let data = syn::Ident::new("data", name.span());
        let original_type = &f.original_type;
        let name_string = name.to_string();
        let err_handler_start = quote! {
            match
        };
        let err_handler_end = quote! {
            {
                Ok(data) => data,
                Err(err) => return Err(err.context(#serialization_mod::DeserializationErrorStackItem::Field(#name_string))),
            }
        };
        let modifier_body = f.modifiers
            .iter()
            .rfold(quote!(#data), |stream, modifier| match modifier {
                Modifier::Min(num) => {
                    quote! {
                        let #data: #original_type = #err_handler_start #serialization_mod::ApplyMin::apply(#data, #num) #err_handler_end;
                        #stream
                    }
                }
                Modifier::Max(num) => {
                    quote! {
                        let #data: #original_type = #err_handler_start #serialization_mod::ApplyMax::apply(#data, #num) #err_handler_end;
                        #stream
                    }
                }
            });
        quote_spanned! { original_type.span()=>
            let #name = {
                let #data: #original_type = #err_handler_start #serialization_mod::ModelDeserializable::<#original_type>::deserialize(#serialized_field_name.#name, registry) #err_handler_end;
                #modifier_body
            };
        }
    });

    let deserialization_impl = quote! {
        #[automatically_derived]
        impl #serialization_mod::ModelDeserializable<#model_name> for #serialized_name {
            fn deserialize(self, registry: &mut #model_mod::PartialModRegistry) -> Result<#model_name, #serialization_mod::DeserializationError> {
                let #serialized_field_name = self;
                #(#modifiers)*

                Ok(#model_name {
                    #(#names),*
                })
            }
        }
    };

    let all_together = quote! {
        #data

        #serialized_struct

        #deserialization_impl
    };

    Ok(all_together.into())
}
