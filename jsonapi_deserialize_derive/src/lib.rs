use darling::{ast, FromDeriveInput, FromField, FromMeta};
use heck::{ToLowerCamelCase, ToPascalCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Generics, Type};

#[proc_macro_derive(JsonApiDeserialize, attributes(json_api))]
pub fn json_api_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_json_api_deserialize(&input).into()
}

#[derive(Debug, FromMeta)]
#[darling(default)]
#[allow(clippy::enum_variant_names)]
enum RenameAll {
    CamelCase,
    PascalCase,
    SnakeCase,
}

impl Default for RenameAll {
    fn default() -> Self {
        Self::CamelCase
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(json_api), supports(struct_any))]
struct InputReceiver {
    ident: Ident,
    #[allow(dead_code)]
    generics: Generics,
    data: ast::Data<(), FieldReceiver>,
    resource_type: Option<String>,
    #[darling(default)]
    rename_all: RenameAll,
}

#[derive(Debug, FromMeta)]
enum Relationship {
    Single,
    Optional,
    Multiple,
}

#[derive(Debug, FromField)]
#[darling(attributes(json_api))]
struct FieldReceiver {
    ident: Option<Ident>,
    #[allow(dead_code)]
    ty: Type,
    relationship: Option<Relationship>,
    resource: Option<Type>,
    rename: Option<String>,
    #[darling(default)]
    default: bool,
    #[darling(default)]
    optional: bool,
}

fn get_attribute_tokens(
    field_name: &Ident,
    json_field_name: &str,
    default: bool,
    optional: bool,
) -> proc_macro2::TokenStream {
    if !(default || optional) {
        return quote! {
            let #field_name = serde_json::from_value(
                data
                    .get("attributes")
                    .ok_or(Error::MissingAttributes)?
                    .get(#json_field_name)
                    .ok_or(Error::MissingField(stringify!(#field_name)))?
                    .clone(),
            )?;
        };
    }

    let mut tokens = quote! {
        let #field_name = data
            .get("attributes")
            .and_then(|attrs| attrs.get(#json_field_name))
            .cloned();
    };

    if default {
        tokens.extend(quote! {
            let #field_name = match #field_name {
                Some(value) => serde_json::from_value(value)?,
                None => Default::default(),
            };
        });
    } else {
        tokens.extend(quote! {
            let #field_name = match #field_name {
                Some(value) => Some(serde_json::from_value(value)?),
                None => None,
            };
        });
    }

    tokens
}

fn get_relationship_tokens(
    field_name: &Ident,
    json_field_name: &str,
    relationship_type: &str,
    default: bool,
    optional: bool,
    lookup_tokens: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let ty = format_ident!("{}", relationship_type);
    let ty = quote! { jsonapi_deserialize::#ty };

    if !(default || optional) {
        return quote! {
            let #field_name = serde_json::from_value::<#ty>(
                data
                    .get("relationships")
                    .ok_or(Error::MissingRelationships)?
                    .get(#json_field_name)
                    .ok_or(Error::MissingField(stringify!(#field_name)))?
                    .clone(),
            )?.data;

            #lookup_tokens
        };
    }

    let mut tokens = quote! {
        let #field_name = data
            .get("relationships")
            .and_then(|attrs| attrs.get(#json_field_name))
            .cloned();
    };

    if default {
        tokens.extend(quote! {
            let #field_name = match #field_name {
                Some(value) => {
                    let #field_name = serde_json::from_value::<#ty>(value)?.data;
                    #lookup_tokens
                    #field_name.into()
                },
                None => Default::default(),
            };
        });
    } else {
        tokens.extend(quote! {
            let #field_name = match #field_name {
                Some(value) => {
                    let #field_name = serde_json::from_value::<#ty>(value)?.data;
                    #lookup_tokens
                    Some(#field_name)
                },
                None => None,
            };
        });
    }

    tokens
}

fn impl_json_api_deserialize(input: &DeriveInput) -> proc_macro2::TokenStream {
    let input_receiver = InputReceiver::from_derive_input(input).unwrap();
    let struct_name = input_receiver.ident;
    let resource_type = input_receiver
        .resource_type
        .unwrap_or_else(|| struct_name.to_string().to_snake_case());

    let mut field_initializers = proc_macro2::TokenStream::new();
    let mut fields = proc_macro2::TokenStream::new();

    input_receiver.data.map_struct_fields(|field| {
        let field_name = match field.ident {
            Some(field_name) => field_name,
            None => return,
        };

        let json_field_name = match field.rename {
            Some(rename) => rename,
            None => match input_receiver.rename_all {
                RenameAll::CamelCase => field_name.to_string().to_lower_camel_case(),
                RenameAll::PascalCase => field_name.to_string().to_pascal_case(),
                RenameAll::SnakeCase => field_name.to_string().to_snake_case(),
            },
        };

        let default = field.default;
        let optional = field.optional;

        let field_tokens = match field.relationship {
            Some(Relationship::Single) => {
                get_relationship_tokens(
                    &field_name,
                    &json_field_name,
                    "RawSingleRelationship",
                    default,
                    optional,
                    field.resource.map(|resource| quote! {
                        let #field_name = included_map.get::<#resource>(&#field_name.kind, &#field_name.id)?;
                    }),
                )
            }
            Some(Relationship::Optional) => {
                get_relationship_tokens(
                    &field_name,
                    &json_field_name,
                    "RawOptionalRelationship",
                    default,
                    optional,
                    field.resource.map(|resource| quote! {
                        let #field_name = match #field_name {
                            Some(data) => Some(included_map.get::<#resource>(&data.kind, &data.id)?),
                            None => None,
                        };
                    }),
                )
            }
            Some(Relationship::Multiple) => {
                get_relationship_tokens(
                    &field_name,
                    &json_field_name,
                    "RawMultipleRelationship",
                    default,
                    optional,
                    field.resource.map(|resource| quote! {
                        let #field_name = #field_name
                            .into_iter()
                            .map(|data| included_map.get::<#resource>(&data.kind, &data.id))
                            .collect::<Result<_, _>>()?;
                    }),
                )
            }
            None => {
                if field_name == "id" {
                    quote! {
                        let #field_name = serde_json::from_value(
                            data
                                .get("id")
                                .ok_or_else(|| Error::MissingId)?
                                .clone(),
                        )?;
                    }
                } else {
                    get_attribute_tokens(&field_name, &json_field_name, default, optional)
                }
            }
        };

        field_initializers.extend(field_tokens);
        fields.extend(quote! { #field_name, });
    });

    let gc_lifetime = quote!{'gc};
    let (struct_generics, static_generic) = if input_receiver.generics.params.is_empty() {
        (quote! {}, quote!{})
    } else {
        (quote!{<'gc>}, quote!{<'static>})
    };

    quote! {
        impl<#gc_lifetime> jsonapi_deserialize::JsonApiDeserialize<#gc_lifetime> for #struct_name #struct_generics {
            type ErasedLifetime = #struct_name #static_generic;
            fn from_value(
                value: &serde_json::Value,
                included_map: &mut jsonapi_deserialize::IncludedMap<'_, #gc_lifetime>,
            ) -> Result<Self, jsonapi_deserialize::DeserializeError> {
                use jsonapi_deserialize::DeserializeError as Error;

                let data = value.as_object().ok_or(Error::InvalidType("Expected an object"))?;

                let resource_type: String = serde_json::from_value(
                    data
                        .get("type")
                        .ok_or_else(|| Error::MissingResourceType)?
                        .clone(),
                )?;

                if resource_type != #resource_type {
                    return Err(Error::ResourceTypeMismatch {
                        expected: #resource_type.to_string(),
                        found: resource_type,
                    });
                }

                #field_initializers

                Ok(Self {
                    #fields
                })
            }
            fn stub() -> Self {
                Default::default()
            }
        }
    }
}
