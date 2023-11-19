use proc_macro::TokenStream;
use std::hash::{BuildHasher, BuildHasherDefault};

use attribute_derive::Attribute;
use convert_case::{Case, Casing};
use proc_macro2::{Ident, Literal};

use quote::{format_ident, quote, quote_spanned};
use rustc_hash::FxHasher;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Error, GenericArgument, ItemStruct, PathArguments, Type};

#[derive(Debug)]
struct FieldData {
    name: Ident,
    original_type: Type,
    definition: proc_macro2::TokenStream,
    modifiers: Vec<Modifier>,
}

#[derive(Debug)]
enum Modifier {
    Min(Literal),
    Max(Literal),
}

#[derive(Debug, Attribute)]
struct AttributeInput {
    name: Option<String>,
}

#[derive(Debug, Attribute)]
#[attribute(ident = model)]
struct FieldAttributeInput {
    min: Option<Literal>,
    max: Option<Literal>,
    id: bool,
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

fn extract_generic(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Paren(ty) => extract_generic(&ty.elem),
        Type::Path(path) => {
            let end = path.path.segments.last()?;
            let PathArguments::AngleBracketed(args) = &end.arguments else {
                return None;
            };
            let Some(GenericArgument::Type(arg)) = args.args.first() else {
                return None;
            };

            Some(arg)
        }
        _ => None,
    }
}

fn serialized_type(
    ty: &Type,
    _modifiers: &mut Vec<Modifier>,
) -> Result<proc_macro2::TokenStream, Error> {
    match ty {
        Type::Paren(ty) => return serialized_type(&ty.elem, _modifiers),
        Type::Path(p) => {
            let Some(end) = p.path.segments.last() else {
                return Err(Error::new(p.span(), "Empty path type"));
            };
            let name = end.ident.to_string();
            match name.as_str() {
                "SlabMapId" => {
                    let Some(_) = extract_generic(ty) else {
                        return Err(Error::new(
                            ty.span(),
                            "Missing generic argument for SlabMapId",
                        ));
                    };
                    return Ok(quote_spanned!(ty.span() => crate::model::ItemId));
                }
                _ => {
                    let end_name = match end.ident.to_string().as_str() {
                        "Handle" => return Ok(quote_spanned! {ty.span()=>String}),
                        _ => {
                            let ident = &end.ident;
                            quote! {#ident}
                        }
                    };
                    let PathArguments::AngleBracketed(args) = &end.arguments else {
                        return Ok(quote! {#ty});
                    };

                    let out_args = args
                        .args
                        .iter()
                        .map(|arg| match arg {
                            GenericArgument::Type(ty) => serialized_type(ty, _modifiers),
                            arg => Ok(quote! {#arg}),
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let mut out_ty: Vec<_> = p.path.segments.iter().collect();
                    let _ = out_ty.pop();
                    let out_ty: Vec<_> = out_ty.into_iter().map(|_e| quote! {e}).collect();
                    return Ok(quote_spanned! {ty.span()=>#(#out_ty::)*#end_name<#(#out_args),*>});
                }
            }
        }
        _ => {}
    }
    Ok(quote! {#ty})
}

fn process(attr: TokenStream, mut data: ItemStruct) -> Result<TokenStream, Error> {
    let mut fields = Vec::new();

    let serialization_mod = quote!(crate::model::serialization);
    let model_mod = quote!(crate::model);
    let mut id_field = None;

    let attr = AttributeInput::from_args(attr.into())?;

    for field in &mut data.fields {
        let Some(name) = &field.ident else {
            return Err(Error::new(field.span(), "All model fields must be named"));
        };
        let ty = &field.ty;

        let attribute_data = FieldAttributeInput::remove_attributes(&mut field.attrs)?;

        let mut modifiers = Vec::new();

        let serialized_type = if let Some(ty) = &attribute_data.ty {
            quote!(#ty)
        } else {
            serialized_type(ty, &mut modifiers)?
        };
        let definition = quote_spanned!(field.span()=>#name: #serialized_type);

        let mut field_data = FieldData {
            name: name.clone(),
            modifiers,
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

    let model_name = &data.ident;
    let serialized_name = format_ident!("{}Serialized", data.ident);

    let tokens = fields.iter().map(|e| &e.definition);
    let serialized_struct = quote!(
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct #serialized_name {
            #(#tokens),*
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
        quote! {
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

            impl #serialization_mod::ModelDeserializable<#model_mod::SlabMapId<#model_name>> for #serialized_name {
                fn deserialize(self, registry: &mut #model_mod::PartialModRegistry) -> Result<#model_mod::SlabMapId<#model_name>, #serialization_mod::DeserializationError> {
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

            impl #serialization_mod::ModelDeserializable<#model_mod::SlabMapId<#model_name>> for #model_mod::ItemId {
                fn deserialize(
                    self,
                    registry: &mut #model_mod::PartialModRegistry,
                ) -> Result<#model_mod::SlabMapId<#model_name>, #serialization_mod::DeserializationError> {
                    if let Some(id) = #serialization_mod::get_reserved_key(&mut registry.#map_name, &self) {
                        return Ok(id)
                    }
                    let Some(other) = registry.raw.#map_name.remove(&self) else {
                        return Err(#serialization_mod::DeserializationErrorKind::MissingItem(self, #model_mod::DatabaseItemKind::#kind_name).into());
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

#[proc_macro_attribute]
pub fn database_model(attr: TokenStream, input: TokenStream) -> TokenStream {
    let data: ItemStruct = parse_macro_input!(input);
    match process(attr, data) {
        Ok(data) => data,
        Err(err) => err.to_compile_error().into(),
    }
}
