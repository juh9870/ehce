use crate::{fallthrough, model_mod, serialization_mod, serialized_type, AttributeInput};
use attribute_derive::Attribute;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote, quote_spanned};
use rustc_hash::FxHasher;
use std::hash::{BuildHasher, BuildHasherDefault};
use syn::spanned::Spanned;
use syn::{Error, ItemStruct, Type};

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
    /// Marks field as ID, turning whole marked struct into a database model
    id: bool,
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
    let mut id_field = None;

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
        if attribute_data.id {
            if id_field.is_some() {
                return Err(Error::new(
                    field.span(),
                    "At most one field can be marked by #[model(id)]",
                ));
            }
            id_field = Some(name);
        }
        attribute_data.apply(&mut field_data);

        fields.push(field_data)
    }

    let tokens = fields.iter().map(|e| &e.definition);
    let serialized_struct = quote!(
        #(#model_fallthrough_attrs)*
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #serialized_name {
            #(#tokens),*
        }

        impl #serialization_mod::ModelDeserializableFallbackType for #model_name {
            type Serialized = #serialized_name;
        }
    );

    let map_name = attr.name.unwrap_or_else(|| {
        model_name
            .to_string()
            .from_case(Case::Pascal)
            .to_case(Case::Snake)
    });
    let kind_name = map_name.from_case(Case::Snake).to_case(Case::Pascal);
    let map_name = format_ident!("{}", map_name);
    let kind_name = format_ident!("{}", kind_name);

    let names = fields.iter().map(|e| &e.name);
    let hasher = BuildHasherDefault::<FxHasher>::default();

    let reservation_field_name = format_ident!("reserved_{}__", hasher.hash_one(&data.ident));
    let serialized_field_name = format_ident!("serialized_{}__", hasher.hash_one(&data.ident));

    let modifiers = fields.iter().map(|f| {
        let name = &f.name;
        let data = syn::Ident::new("data", name.span());
        let original_type = &f.original_type;
        let name_string = name.to_string();
        let id_handler = if let Some(id) = id_field {
            quote!(
                .context(#serialization_mod::DeserializationErrorStackItem::Item(#serialized_field_name.#id, #model_mod::DatabaseItemKind::#kind_name))
            )
        } else {
            quote!()
        };
        let err_handler_start = quote! {
            match
        };
        let err_handler_end = quote! {
            {
                Ok(data) => data,
                Err(err) => return Err(err.context(#serialization_mod::DeserializationErrorStackItem::Field(#name_string))#id_handler),
            }
        };
        if !id_field.map(|e|e==name).unwrap_or(false) {
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
        } else {
            quote! {} // assigned at a later date
        }
    });

    let deserialization_impl = if let Some(id_field) = id_field {
        let id_name = format_ident!("{}Id", model_name);
        quote! {

            pub type #id_name = #model_mod::SlabMapId<#model_name>;

            impl #model_mod::DatabaseItemTrait for #model_name {
                fn id(&self) -> utils::slab_map::SlabMapUntypedId {
                    self.id.as_untyped()
                }

                fn kind(&self) -> #model_mod::DatabaseItemKind {
                    #model_mod::DatabaseItemKind::#kind_name
                }
            }

            impl #model_mod::DatabaseModelSerializationHelper for #model_name {
                type Serialized = #serialized_name;
            }

            impl #model_mod::ModelKind for #model_name {
                fn kind() -> #model_mod::DatabaseItemKind {
                    #model_mod::DatabaseItemKind::#kind_name
                }
            }

            impl #model_mod::ModelKind for #serialized_name {
                fn kind() -> #model_mod::DatabaseItemKind {
                    #model_mod::DatabaseItemKind::#kind_name
                }
            }

            impl #model_mod::DatabaseItemSerializedTrait for #serialized_name {
                fn id(&self) -> &#model_mod::ItemId {
                    &self.id
                }

                fn kind(&self) -> #model_mod::DatabaseItemKind {
                    #model_mod::DatabaseItemKind::#kind_name
                }
            }

            impl #serialization_mod::ModelDeserializable<#id_name> for #serialized_name {
                fn deserialize(self, registry: &mut #model_mod::PartialModRegistry) -> Result<#id_name, #serialization_mod::DeserializationError> {
                    let #serialized_field_name = self;
                    let #reservation_field_name = #serialization_mod::reserve(&mut registry.#map_name, #serialized_field_name.#id_field.clone())?;

                    #(#modifiers)*

                    let #id_field = #reservation_field_name.raw();
                    let model = #model_name {
                        #(#names),*
                    };
                    let id = #serialization_mod::insert_reserved(&mut registry.#map_name, #reservation_field_name, model);

                    Ok(id)
                }
            }

            impl #serialization_mod::ModelDeserializable<#id_name> for &str {
                fn deserialize(
                    self,
                    registry: &mut #model_mod::PartialModRegistry,
                ) -> Result<#id_name, #serialization_mod::DeserializationError> {
                    if let Some(id) = #serialization_mod::get_reserved_key(&mut registry.#map_name, self) {
                        return Ok(id)
                    }
                    let Some(other) = registry.raw.#map_name.remove(self) else {
                        return Err(#serialization_mod::DeserializationErrorKind::MissingItem(self.to_string(), #model_mod::DatabaseItemKind::#kind_name).into());
                    };

                    other.deserialize(registry)
                }
            }
        }
    } else {
        quote! {
            impl #serialization_mod::ModelDeserializable<#model_name> for #serialized_name {
                fn deserialize(self, registry: &mut #model_mod::PartialModRegistry) -> Result<#model_name, #serialization_mod::DeserializationError> {
                    let #serialized_field_name = self;
                    #(#modifiers)*

                    Ok(#model_name {
                        #(#names),*
                    })
                }
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
