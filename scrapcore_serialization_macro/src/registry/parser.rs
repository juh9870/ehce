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
}

#[derive(Debug, Attribute)]
#[attribute(ident = model)]
struct ModelAttributeInput {
    #[attribute(conflicts=[registry, singleton])]
    asset: bool,
    #[attribute(conflicts=[asset, singleton])]
    registry: bool,
    #[attribute(conflicts=[asset, registry])]
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
    let mut registry = RegistryDefinitions {
        pascal_name: registry_item_name.clone(),
        serialized_model_name: input
            .serialized_item_name
            .unwrap_or_else(|| format_ident!("{registry_item_name}Serialized")),
        schema: input.schema,
        singletons: Default::default(),
        registries: Default::default(),
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
                variant_name: Ident::new(
                    &name
                        .to_string()
                        .from_case(Case::Snake)
                        .to_case(Case::Pascal),
                    field.span(),
                ),
                field_name: name,
                ty: field.ty.clone(),
                ty_serialized: serialized_of(&field.ty)?,
            };
            if attribute.registry {
                registry.registries.0.push(model);
            } else if attribute.singleton {
                registry.singletons.0.push(model);
            } else {
                return Err(syn::Error::new(
                    field.span(),
                    "All fields must be annotated with #[model(asset)], #[model(registry)], or #[model(singleton)]",
                ));
            }
        }
    }
    Ok(registry)
}
