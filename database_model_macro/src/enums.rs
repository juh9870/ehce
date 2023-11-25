use attribute_derive::Attribute;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Error, Fields, ItemEnum, Type};

use crate::{fallthrough, model_mod, serialization_mod, serialized_type, AttributeInput};

#[derive(Debug, Attribute)]
#[attribute(ident = model)]
struct EnumVariantAttributeInput {
    ty: Option<Type>,
}

pub fn process_enum(
    attr: proc_macro::TokenStream,
    mut data: ItemEnum,
) -> Result<proc_macro::TokenStream, Error> {
    let attr = AttributeInput::from_args(attr.into())?;
    let serialization_mod = serialization_mod();
    let model_mod = model_mod();
    let model_name = &data.ident;
    let serialized_name = attr
        .name
        .as_ref()
        .map(|e| format_ident!("{e}"))
        .unwrap_or_else(|| format_ident!("{}Serialized", data.ident));

    let variants = data.variants.iter_mut().map(|variant| {
        let input = EnumVariantAttributeInput::remove_attributes(&mut variant.attrs)?;

        let (original_ty, serialized_ty) = if let Fields::Unnamed(field) = &variant.fields {
            let item = field.unnamed.iter().exactly_one().map_err(|_e| {
                Error::new(field.span(), "Enum variant must have exactly one field")
            })?;
            let ty = if let Some(ty) = &input.ty {
                quote!(#ty)
            } else {
                serialized_type(&item.ty)?
            };
            (&item.ty, ty)
        } else {
            return Err(Error::new(
                variant.span(),
                "Only newtype enums are supported",
            ));
        };


        let variant_name = &variant.ident;

        let fallthrough_attrs = fallthrough(&mut variant.attrs);

        let serialized_variant = quote_spanned! {variant.span()=>
            #(#fallthrough_attrs)*
            #variant_name(#serialized_ty),
        };
        let deserialization_match = quote_spanned! {variant.span()=>
            Self::#variant_name(item) => #model_name::#variant_name(#serialization_mod::ModelDeserializable::<#original_ty>::deserialize(item, registry)?),
        };

        Result::<(TokenStream, TokenStream), Error>::Ok((serialized_variant, deserialization_match))
    }).collect::<Result<Vec<(TokenStream, TokenStream)>,_>>()?;

    let (variants, deserialization): (Vec<_>, Vec<_>) = variants.into_iter().unzip();

    let model_name_str = model_name.to_string();
    let schema_derive = attr.schema_derive();
    Ok(quote! {
        #data

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename = #model_name_str)]
        #schema_derive
        pub enum #serialized_name {
            #(#variants)*
        }

        #[automatically_derived]
        impl #serialization_mod::ModelDeserializableFallbackType for #model_name {
            type Serialized = #serialized_name;
        }

        #[automatically_derived]
        impl #serialization_mod::ModelDeserializable<#model_name> for #serialized_name {
            fn deserialize(self, registry: &mut #model_mod::PartialModRegistry) -> Result<#model_name, #serialization_mod::DeserializationError> {
                Ok(match self {
                    #(#deserialization)*
                })
            }
        }
    }
    .into())
}
