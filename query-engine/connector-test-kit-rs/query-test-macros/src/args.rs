use crate::IntoDarlingError;
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use query_tests_setup::{relation_field::RelationField, ConnectorTag};
use quote::quote;
use std::convert::TryFrom;
use syn::{spanned::Spanned, Ident, Meta, Path};

#[derive(Debug, FromMeta)]
pub struct ConnectorTestArgs {
    #[darling(default)]
    pub suite: Option<String>,

    #[darling(default)]
    pub schema: Option<SchemaHandler>,

    #[darling(default)]
    pub only: OnlyConnectorTags,

    #[darling(default)]
    pub exclude: ExcludeConnectorTags,

    #[darling(default)]
    pub capabilities: RunOnlyForCapabilities,
}

impl ConnectorTestArgs {
    pub fn validate(&self, on_module: bool) -> Result<(), darling::Error> {
        if !self.only.is_empty() && !self.exclude.is_empty() && !on_module {
            return Err(darling::Error::custom(
                "Only one of `only` and `exclude` can be specified for a connector test.",
            ));
        }

        if self.schema.is_none() && !on_module {
            return Err(darling::Error::custom(
                "A schema annotation on either the test mod (#[test_suite(schema(handler))]) or the test (schema(handler)) is required.",
            ));
        }

        if self.suite.is_none() && !on_module {
            return Err(darling::Error::custom(
                "A test suite name annotation on either the test mod (#[test_suite]) or the test (suite = \"name\") is required.",
            ));
        }

        Ok(())
    }

    /// Returns all the connectors that the test is valid for.
    pub fn connectors_to_test(&self) -> Vec<ConnectorTag> {
        if !self.only.is_empty() {
            self.only.tags.clone()
        } else if !self.exclude.is_empty() {
            let all = ConnectorTag::all();
            let exclude = self.exclude.tags();

            all.into_iter().filter(|tag| !exclude.contains(tag)).collect()
        } else {
            ConnectorTag::all()
        }
    }
}

#[derive(Debug)]
pub struct SchemaHandler {
    pub handler_path: Path,
}

impl darling::FromMeta for SchemaHandler {
    fn from_list(items: &[syn::NestedMeta]) -> Result<Self, darling::Error> {
        if items.len() != 1 {
            return Err(darling::Error::unsupported_shape(
                "Expected `schema` to contain exactly one function pointer to a schema handler.",
            )
            .with_span(&Span::call_site()));
        }

        let item = items.first().unwrap();
        match item {
            syn::NestedMeta::Meta(Meta::Path(p)) => Ok(Self {
                // Todo validate signature somehow
                handler_path: p.clone(),
            }),
            x => Err(darling::Error::unsupported_shape(
                "Expected `schema` to be a function pointer to a schema handler function.",
            )
            .with_span(&x.span())),
        }
    }
}

#[derive(Debug, Default)]
pub struct OnlyConnectorTags {
    tags: Vec<ConnectorTag>,
    token_stream: TokenStream,
}

impl OnlyConnectorTags {
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct ExcludeConnectorTags {
    tags: Vec<ConnectorTag>,
}

impl ExcludeConnectorTags {
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn tags(&self) -> &[ConnectorTag] {
        &self.tags
    }
}

impl darling::FromMeta for OnlyConnectorTags {
    fn from_list(items: &[syn::NestedMeta]) -> Result<Self, darling::Error> {
        let token_stream = quote! { #(#items),* }.into();
        let tags = tags_from_list(items)?;

        Ok(OnlyConnectorTags { tags, token_stream })
    }
}

impl darling::FromMeta for ExcludeConnectorTags {
    fn from_list(items: &[syn::NestedMeta]) -> Result<Self, darling::Error> {
        let tags = tags_from_list(items)?;
        Ok(ExcludeConnectorTags { tags })
    }
}

fn tags_from_list(items: &[syn::NestedMeta]) -> Result<Vec<ConnectorTag>, darling::Error> {
    if items.is_empty() {
        return Err(darling::Error::custom("At least one connector tag is required."));
    }

    let mut tags: Vec<ConnectorTag> = vec![];

    for item in items {
        match item {
            syn::NestedMeta::Meta(meta) => {
                match meta {
                    // A single variant without version, like `Postgres`.
                    Meta::Path(p) => {
                        let tag = tag_string_from_path(p)?;
                        tags.push(ConnectorTag::try_from(tag.as_str()).into_darling_error(&p.span())?);
                    }
                    Meta::List(l) => {
                        let tag = tag_string_from_path(&l.path)?;
                        for meta in l.nested.iter() {
                            match meta {
                                syn::NestedMeta::Lit(literal) => {
                                    let version_str = match literal {
                                        syn::Lit::Str(s) => s.value(),
                                        syn::Lit::Char(c) => c.value().to_string(),
                                        syn::Lit::Int(i) => i.to_string(),
                                        syn::Lit::Float(f) => f.to_string(),
                                        x => {
                                            return Err(darling::Error::unexpected_type(
                                                "Versions can be string, char, int and float.",
                                            )
                                            .with_span(&x.span()))
                                        }
                                    };

                                    tags.push(
                                        ConnectorTag::try_from((tag.as_str(), Some(version_str.as_str())))
                                            .into_darling_error(&l.span())?,
                                    );
                                }
                                syn::NestedMeta::Meta(meta) => {
                                    return Err(darling::Error::unexpected_type(
                                        "Versions can only be literals (string, char, int and float).",
                                    )
                                    .with_span(&meta.span()));
                                }
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            x => {
                return Err(
                    darling::Error::custom("Expected `only` or `exclude` to be a list of `ConnectorTag`.")
                        .with_span(&x.span()),
                )
            }
        }
    }

    Ok(tags)
}

fn tag_string_from_path(path: &Path) -> Result<String, darling::Error> {
    if let Some(ident) = path.get_ident() {
        let name = ident.to_string();

        Ok(name)
    } else {
        Err(darling::Error::custom(
            "Expected `only` to be a list of idents (ConnectorTag variants), not paths.",
        ))
    }
}

#[derive(Debug, Default)]
pub struct RunOnlyForCapabilities {
    pub idents: Vec<Ident>,
}

impl darling::FromMeta for RunOnlyForCapabilities {
    fn from_list(items: &[syn::NestedMeta]) -> Result<Self, darling::Error> {
        if items.is_empty() {
            return Err(darling::Error::custom(
                "When specifying capabilities to run for, at least one needs to be given.",
            ));
        }

        let mut idents: Vec<Ident> = vec![];

        for item in items {
            match item {
                syn::NestedMeta::Meta(meta) => {
                    match meta {
                        // A single variant without version, like `Postgres`.
                        Meta::Path(p) => match p.get_ident() {
                            Some(ident) => idents.push(ident.clone()),
                            None => {
                                return Err(darling::Error::unexpected_type("Invalid identifier").with_span(&p.span()))
                            }
                        },
                        x => return Err(darling::Error::unexpected_type("Expected identifiers").with_span(&x.span())),
                    }
                }
                x => {
                    return Err(
                        darling::Error::custom("Expected `only` or `exclude` to be a list of `ConnectorTag`.")
                            .with_span(&x.span()),
                    )
                }
            }
        }

        Ok(Self { idents })
    }
}

#[derive(Debug, FromMeta)]
pub struct ConnectorTestGenArgs {
    #[darling(default)]
    pub suite: Option<String>,

    #[darling(default)]
    pub only: OnlyConnectorTags,

    // TODO: Find a better name
    pub gen: SchemaGen,

    #[darling(default)]
    pub exclude: ExcludeConnectorTags,

    #[darling(default)]
    pub capabilities: RunOnlyForCapabilities,
}

impl ConnectorTestGenArgs {
    pub fn validate(&self, on_module: bool) -> Result<(), darling::Error> {
        if !self.only.is_empty() && !self.exclude.is_empty() && !on_module {
            return Err(darling::Error::custom(
                "Only one of `only` and `exclude` can be specified for a connector test.",
            ));
        }

        if self.suite.is_none() && !on_module {
            return Err(darling::Error::custom(
                "A test suite name annotation on either the test mod (#[test_suite]) or the test (suite = \"name\") is required.",
            ));
        }

        Ok(())
    }

    /// Returns all the connectors that the test is valid for.
    pub fn connectors_to_test(&self) -> Vec<ConnectorTag> {
        if !self.only.is_empty() {
            self.only.tags.clone()
        } else if !self.exclude.is_empty() {
            let all = ConnectorTag::all();
            let exclude = self.exclude.tags();

            all.into_iter().filter(|tag| !exclude.contains(tag)).collect()
        } else {
            ConnectorTag::all()
        }
    }
}

#[derive(Debug)]
pub struct SchemaGen {
    pub on_parent: RelationField,
    pub on_child: RelationField,
    pub without_parent: bool,
}

impl darling::FromMeta for SchemaGen {
    fn from_list(items: &[syn::NestedMeta]) -> Result<Self, darling::Error> {
        if items.len() < 2 {
            return Err(darling::Error::unsupported_shape(
                "Expected `gen` to contain at least 2 RelationField and an optional third param boolean field",
            )
            .with_span(&Span::call_site()));
        }

        let mut items_iter = items.iter();

        let on_parent = get_next_relation_field(&mut items_iter, "first")?;
        let on_child = get_next_relation_field(&mut items_iter, "second")?;

        // Accepts both x = <bool> or <bool>
        // If no value provided, defaults to false
        let without_parent = match items_iter.next() {
            Some(next) => match next {
                syn::NestedMeta::Meta(Meta::NameValue(name_value)) => match &name_value.lit {
                    syn::Lit::Bool(b) => Ok(b.value),
                    x => Err(darling::Error::unsupported_shape(
                        "Expected `gen` third param to be a NameValue of value boolean (eg: `without_params = <bool>`) or just a boolean `<bool>`",
                    )
                    .with_span(&x.span())),
                },
                syn::NestedMeta::Lit(syn::Lit::Bool(b)) => Ok(b.value),
                x => Err(darling::Error::unsupported_shape(
                    "Expected `gen` third param to be a NameValue of value boolean (eg: `without_params = <bool>`) or just a boolean `<bool>`",
                )
                .with_span(&x.span())),
            }?,
            None => false,
        };

        Ok(Self {
            on_child,
            on_parent,
            without_parent,
        })
    }
}

fn get_next_relation_field(
    items: &mut std::slice::Iter<syn::NestedMeta>,
    position: &str,
) -> Result<RelationField, darling::Error> {
    let err: darling::Error = darling::Error::custom(format!(
        "Expected `gen` {} param to be a list of idents (RelationField variants). eg: ParentReq",
        position
    ));

    match items.next().unwrap() {
        syn::NestedMeta::Meta(Meta::Path(p)) => {
            let tag = if let Some(ident) = p.get_ident() {
                let name = ident.to_string();
                Ok(name)
            } else {
                Err(err)
            }?;

            RelationField::try_from(tag.as_str()).into_darling_error(&p.span())
        }
        _ => Err(err),
    }
}
