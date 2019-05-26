// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use quote::quote;
use syn::{parse::Result as ParseResult, spanned::Spanned, Error as SynError};

const ATTR_NAME: &str = "property";

pub(crate) struct PropertyDef {
    pub(crate) name: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) fields: Vec<FieldDef>,
}

pub(crate) struct FieldDef {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
    pub(crate) conf: FieldConf,
}

#[derive(Clone)]
pub(crate) enum GetTypeConf {
    NotSet,
    Ref,
    Copy_,
    Clone_,
}

#[derive(Clone)]
pub(crate) enum FieldVisConf {
    Disable,
    Public,
    Crate,
    Private,
}

#[derive(Clone)]
pub(crate) struct FieldConf {
    pub(crate) get: FieldVisConf,
    pub(crate) set: FieldVisConf,
    pub(crate) mut_: FieldVisConf,
    pub(crate) get_type: GetTypeConf,
}

impl syn::parse::Parse for PropertyDef {
    fn parse(input: syn::parse::ParseStream) -> ParseResult<Self> {
        let derive_input: syn::DeriveInput = input.parse()?;
        let span = derive_input.span();
        let syn::DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = derive_input;
        let conf = Self::parse_attrs(span, &attrs[..])?;
        Ok(Self {
            name: ident,
            generics,
            fields: FieldDef::parse_data(data, conf, span)?,
        })
    }
}

impl PropertyDef {
    fn parse_attrs(span: proc_macro2::Span, attrs: &[syn::Attribute]) -> ParseResult<FieldConf> {
        Ok(parse_attrs(span, Default::default(), attrs)?)
    }
}

impl FieldDef {
    fn parse_data(
        data: syn::Data,
        conf: FieldConf,
        span: proc_macro2::Span,
    ) -> ParseResult<Vec<Self>> {
        match data {
            syn::Data::Struct(data) => {
                let mut fields = Vec::new();
                for f in data.fields.into_iter() {
                    let syn::Field {
                        attrs, ident, ty, ..
                    } = f.clone();
                    let conf = Self::parse_attrs(f.span(), conf.clone(), &attrs[..])?;
                    let ident =
                        ident.ok_or_else(|| SynError::new(f.span(), "only support named field"))?;
                    let field = Self { ident, ty, conf };
                    fields.push(field);
                }
                Ok(fields)
            }
            _ => Err(SynError::new(
                span,
                "`#[derive(Property)]` only support structs",
            )),
        }
    }

    fn parse_attrs(
        span: proc_macro2::Span,
        conf: FieldConf,
        attrs: &[syn::Attribute],
    ) -> ParseResult<FieldConf> {
        Ok(parse_attrs(span, conf, attrs)?)
    }
}

impl GetTypeConf {
    pub(crate) fn parse_from_input(
        params: &::std::collections::HashSet<&syn::Ident>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let result = check_params(params, &[&["copy", "clone", "ref"]])?;
        let choice = match result[0] {
            None => None,
            Some("clone") => Some(GetTypeConf::Clone_),
            Some("copy") => Some(GetTypeConf::Copy_),
            Some("ref") => Some(GetTypeConf::Ref),
            _ => Err(SynError::new(span, "unreachable result"))?,
        };
        Ok(choice)
    }
}

impl FieldVisConf {
    pub(crate) fn parse_from_input(
        params: &::std::collections::HashSet<&syn::Ident>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let result = check_params(params, &[&["disable", "public", "crate", "private"]])?;
        let choice = match result[0] {
            None => None,
            Some("disable") => Some(FieldVisConf::Disable),
            Some("public") => Some(FieldVisConf::Public),
            Some("crate") => Some(FieldVisConf::Crate),
            Some("private") => Some(FieldVisConf::Private),
            _ => Err(SynError::new(span, "unreachable result"))?,
        };
        Ok(choice)
    }

    pub(crate) fn to_visibility(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            FieldVisConf::Disable => None,
            FieldVisConf::Public => Some(quote!(pub)),
            FieldVisConf::Crate => Some(quote!(pub(crate))),
            FieldVisConf::Private => Some(quote!()),
        }
    }
}

impl ::std::default::Default for FieldConf {
    fn default() -> Self {
        Self {
            get: FieldVisConf::Crate,
            set: FieldVisConf::Crate,
            mut_: FieldVisConf::Crate,
            get_type: GetTypeConf::NotSet,
        }
    }
}

impl FieldConf {
    fn apply_attrs(&mut self, meta: &syn::Meta) -> ParseResult<()> {
        match meta {
            syn::Meta::Word(ident) => {
                Err(SynError::new(
                    ident.span(),
                    "this attribute should not be a word",
                ))?;
            }
            syn::Meta::List(list) => {
                let mut params = ::std::collections::HashSet::new();
                for nested_meta in list.nested.iter() {
                    match nested_meta {
                        syn::NestedMeta::Meta(meta) => match meta {
                            syn::Meta::Word(ident) => {
                                if !params.insert(ident) {
                                    Err(SynError::new(
                                        ident.span(),
                                        "this attribute has been set twice",
                                    ))?;
                                }
                            }
                            _ => {
                                Err(SynError::new(
                                    meta.span(),
                                    "this attribute should be a word",
                                ))?;
                            }
                        },
                        syn::NestedMeta::Literal(lit) => {
                            Err(SynError::new(
                                lit.span(),
                                "this attribute should not be a literal",
                            ))?;
                        }
                    }
                }
                if params.is_empty() {
                    Err(SynError::new(
                        list.span(),
                        "this attribute should not be empty",
                    ))?;
                }
                match list.ident.to_string().as_ref() {
                    "get" => {
                        if let Some(choice) =
                            FieldVisConf::parse_from_input(&params, list.ident.span())?
                        {
                            self.get = choice;
                        }
                    }
                    "set" => {
                        if let Some(choice) =
                            FieldVisConf::parse_from_input(&params, list.ident.span())?
                        {
                            self.set = choice;
                        }
                    }
                    "mut" => {
                        if let Some(choice) =
                            FieldVisConf::parse_from_input(&params, list.ident.span())?
                        {
                            self.mut_ = choice;
                        }
                    }
                    "get_type" => {
                        if let Some(choice) =
                            GetTypeConf::parse_from_input(&params, list.ident.span())?
                        {
                            self.get_type = choice;
                        }
                    }
                    _ => {
                        Err(SynError::new(list.ident.span(), "unsupport attribute"))?;
                    }
                }
            }
            syn::Meta::NameValue(name_value) => {
                Err(SynError::new(
                    name_value.span(),
                    "this attribute should not be a name-value pair",
                ))?;
            }
        }
        Ok(())
    }
}

fn check_params<'a>(
    params: &::std::collections::HashSet<&syn::Ident>,
    options: &[&[&'a str]],
) -> ParseResult<Vec<Option<&'a str>>> {
    let mut result = vec![None; options.len()];
    let mut find;
    for p in params.iter() {
        find = false;
        for (i, group) in options.iter().enumerate() {
            for opt in group.iter() {
                if p == opt {
                    find = true;
                    if result[i].is_some() {
                        Err(SynError::new(p.span(), "this attribute was set twice"))?;
                    }
                    result[i] = Some(*opt);
                    break;
                }
            }
            if find {
                break;
            }
        }
        if !find {
            Err(SynError::new(p.span(), "this attribute was unknown"))?;
        }
    }
    Ok(result)
}

fn parse_attrs(
    span: proc_macro2::Span,
    mut conf: FieldConf,
    attrs: &[syn::Attribute],
) -> ParseResult<FieldConf> {
    for attr in attrs.iter() {
        if let syn::AttrStyle::Outer = attr.style {
            let meta = attr
                .parse_meta()
                .map_err(|_| SynError::new(span, "failed to parse the attributes"))?;
            match meta {
                syn::Meta::Word(ident) => {
                    if ident == ATTR_NAME {
                        Err(SynError::new(
                            ident.span(),
                            "the attribute should not be a word",
                        ))?;
                    }
                }
                syn::Meta::List(list) => {
                    if list.ident == ATTR_NAME {
                        for nested_meta in list.nested.iter() {
                            match nested_meta {
                                syn::NestedMeta::Meta(meta) => {
                                    conf.apply_attrs(meta)?;
                                }
                                syn::NestedMeta::Literal(lit) => {
                                    Err(SynError::new(
                                        lit.span(),
                                        "the attribute in nested meta should not be a literal",
                                    ))?;
                                }
                            }
                        }
                    }
                }
                syn::Meta::NameValue(name_value) => {
                    if name_value.ident == ATTR_NAME {
                        Err(SynError::new(
                            name_value.span(),
                            "the attribute should not be a name-value pair",
                        ))?;
                    }
                }
            }
        }
    }
    Ok(conf)
}
