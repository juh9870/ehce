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

// #[derive(Debug)]
// struct TypePathWrapper(TypePath);
//
// impl<'de> serde::Deserialize<'de> for TypePathWrapper {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let path = String::deserialize(deserializer)?;
//         let path = match syn::parse_str::<TypePath>(&path) {
//             Ok(data) => data,
//             Err(err) => return Err(D::Error::custom(err.to_string())),
//         };
//         return Ok(Self(path));
//     }
// }

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

// impl<'de> serde::Deserialize<'de> for ModelSet {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let hm = FxHashMap::<String, TypePathWrapper>::deserialize(deserializer)?;
//         Ok(Self(
//             hm.into_iter()
//                 .map(|(k, v)| {
//                     let field_name = k;
//                     if !field_name.is_case(Case::Snake) {
//                         return Err(D::Error::custom("Model name is not in snake case"));
//                     }
//                     let variant_name = field_name.from_case(Case::Snake).to_case(Case::Pascal);
//                     let ty = v.0;
//                     Ok(ModelKind {
//                         field_name,
//                         variant_name: Ident::new(&variant_name, Span::call_site()),
//                         ty_serialized: serialized_of(&ty),
//                         ty,
//                     })
//                 })
//                 .collect::<Result<Vec<ModelKind>, D::Error>>()?,
//         ))
//     }
// }

// #[derive(Debug, Deserialize)]
// struct RawRegistryDefinitions {
//     name: String,
//     serialized_model_name: Option<String>,
//     schema: Option<bool>,
//     /// Singleton values, only one of each item is allowed and required per mod
//     singletons: ModelSet,
//     /// Registries, multiple of each item may be present in a mod
//     registries: ModelSet,
//     /// Assets, loaded as-is with no dependencies
//     assets: FxHashMap<String, String>,
// }

#[derive(Debug)]
struct RegistryDefinitions {
    pascal_name: Ident,
    serialized_model_name: Ident,
    schema: bool,

    singletons: ModelSet,
    registries: ModelSet,
    assets: FxHashMap<Ident, Type>,
}

// impl<'de> serde::Deserialize<'de> for RegistryDefinitions {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let defs = RawRegistryDefinitions::deserialize(deserializer)?;
//         if !defs.name.is_case(Case::Pascal) {
//             return Err(D::Error::custom("`name` is not in PascalCase"));
//         }
//         let serialized_model_name = defs
//             .serialized_model_name
//             .unwrap_or_else(|| format!("{}Serialized", defs.name));
//         if !serialized_model_name.is_case(Case::Pascal) {
//             return Err(D::Error::custom(
//                 "`serialized_model_name` is not in PascalCase",
//             ));
//         }
//         Ok(RegistryDefinitions {
//             pascal_name: defs.name,
//             serialized_model_name: format_ident!("{}", serialized_model_name),
//             schema: defs.schema.unwrap_or(true),
//             singletons: defs.singletons,
//             registries: defs.registries,
//             assets: defs.assets,
//         })
//     }
// }

fn registry_impl_inner(
    attr: proc_macro::TokenStream,
    mut item_struct: ItemStruct,
) -> syn::Result<TokenStream> {
    // let bytes = std::fs::read(input.file.0)
    //     .map_err(|_| syn::Error::new(input.file.1, "Failed to read definitions file"))?;
    // let content = String::from_utf8(bytes)
    //     .map_err(|_| syn::Error::new(input.file.1, "Definitions file is not a valid UTF8 text"))?;

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
