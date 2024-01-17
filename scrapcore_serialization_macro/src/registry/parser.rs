use crate::registry::{serialized_of, ModelKind, RegistryDefinitions};
use attribute_derive::Attribute;
use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::format_ident;
use rustc_hash::FxHashSet;
use syn::spanned::Spanned;
use syn::ItemStruct;

#[derive(Debug, Attribute)]
struct RegistryAttributeInput {
    #[attribute(default = true)]
    schema: bool,
    item_name: Option<Ident>,
    serialized_item_name: Option<Ident>,
    registry_name: Option<Ident>,
    partial_registry_name: Option<Ident>,
}

#[derive(Debug, Attribute)]
#[attribute(ident = model)]
struct ModelAttributeInput {
    #[attribute(conflicts=[collection, singleton])]
    asset: bool,
    #[attribute(conflicts=[asset, singleton])]
    collection: bool,
    #[attribute(conflicts=[asset, collection])]
    singleton: bool,
}
pub(super) fn parse_struct_defs(
    attr: proc_macro::TokenStream,
    data: &mut ItemStruct,
) -> syn::Result<RegistryDefinitions> {
    let mut used_types = FxHashSet::default();
    let input = RegistryAttributeInput::from_args(attr.into())?;
    let registry_item_name = input
        .item_name
        .unwrap_or_else(|| format_ident!("{}Item", data.ident));
    let registry_name = input
        .registry_name
        .unwrap_or_else(|| format_ident!("{}Registry", data.ident));
    let partial_registry_name = input
        .partial_registry_name
        .unwrap_or_else(|| format_ident!("Partial{}", registry_name));
    let mut registry = RegistryDefinitions {
        pascal_name: registry_item_name.clone(),
        serialized_model_name: input
            .serialized_item_name
            .unwrap_or_else(|| format_ident!("{registry_item_name}Serialized")),
        registry_name,
        partial_registry_name,
        model_name: registry_item_name,
        schema: input.schema,
        singletons: Default::default(),
        collections: Default::default(),
        assets: Default::default(),
    };
    for field in &mut data.fields {
        if !used_types.insert(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "This type is already defined in the model",
            ));
        }
        let attribute = ModelAttributeInput::remove_attributes(&mut field.attrs)?;
        let name = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(field.span(), "Tuple enums are not supported"))?;

        if attribute.asset {
            registry.assets.insert(name, field.ty.clone());
        } else {
            let model = ModelKind {
                span: field.span(),
                variant_name: Ident::new(
                    &name
                        .to_string()
                        .from_case(Case::Snake)
                        .to_case(Case::Pascal),
                    field.span(),
                ),
                raw_field_name: format_ident!("{name}_raw"),
                field_name: name,
                ty: field.ty.clone(),
                ty_serialized: serialized_of(&field.ty)?,
            };
            if attribute.collection {
                registry.collections.0.push(model);
            } else if attribute.singleton {
                registry.singletons.0.push(model);
            } else {
                return Err(syn::Error::new(
                    field.span(),
                    "All fields must be annotated with #[model(asset)], #[model(collection)], or #[model(singleton)]",
                ));
            }
        }
    }
    Ok(registry)
}
