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

const GET_TYPE_OPTIONS: (&str, Option<&[&str]>) = ("type", Some(&["ref", "copy", "clone"]));
const VISIBILITY_OPTIONS: &[&str] = &["disable", "public", "crate", "private"];

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
pub(crate) enum VisibilityConf {
    Disable,
    Public,
    Crate,
    Private,
}

#[derive(Clone)]
pub(crate) struct GetFieldConf {
    pub(crate) vis: VisibilityConf,
    pub(crate) typ: GetTypeConf,
}

#[derive(Clone)]
pub(crate) struct SetFieldConf {
    pub(crate) vis: VisibilityConf,
}

#[derive(Clone)]
pub(crate) struct MutFieldConf {
    pub(crate) vis: VisibilityConf,
}

#[derive(Clone)]
pub(crate) struct FieldConf {
    pub(crate) get: GetFieldConf,
    pub(crate) set: SetFieldConf,
    pub(crate) mut_: MutFieldConf,
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
        namevalue_params: &::std::collections::HashMap<&str, String>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let choice = match namevalue_params.get("type").map(AsRef::as_ref) {
            None => None,
            Some("ref") => Some(GetTypeConf::Ref),
            Some("copy") => Some(GetTypeConf::Copy_),
            Some("clone") => Some(GetTypeConf::Clone_),
            _ => Err(SynError::new(span, "unreachable result"))?,
        };
        Ok(choice)
    }
}

impl VisibilityConf {
    pub(crate) fn parse_from_input(
        input: Option<&str>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let choice = match input {
            None => None,
            Some("disable") => Some(VisibilityConf::Disable),
            Some("public") => Some(VisibilityConf::Public),
            Some("crate") => Some(VisibilityConf::Crate),
            Some("private") => Some(VisibilityConf::Private),
            _ => Err(SynError::new(span, "unreachable result"))?,
        };
        Ok(choice)
    }

    pub(crate) fn to_ts(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            VisibilityConf::Disable => None,
            VisibilityConf::Public => Some(quote!(pub)),
            VisibilityConf::Crate => Some(quote!(pub(crate))),
            VisibilityConf::Private => Some(quote!()),
        }
    }
}

impl ::std::default::Default for FieldConf {
    fn default() -> Self {
        Self {
            get: GetFieldConf {
                vis: VisibilityConf::Crate,
                typ: GetTypeConf::NotSet,
            },
            set: SetFieldConf {
                vis: VisibilityConf::Crate,
            },
            mut_: MutFieldConf {
                vis: VisibilityConf::Crate,
            },
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
                let mut word_params = ::std::collections::HashSet::new();
                let mut namevalue_params = ::std::collections::HashMap::new();
                for nested_meta in list.nested.iter() {
                    match nested_meta {
                        syn::NestedMeta::Meta(meta) => match meta {
                            syn::Meta::Word(ident) => {
                                if !word_params.insert(ident) {
                                    Err(SynError::new(
                                        ident.span(),
                                        "this attribute has been set twice",
                                    ))?;
                                }
                            }
                            syn::Meta::NameValue(mnv) => {
                                let syn::MetaNameValue { ident, lit, .. } = mnv;
                                if let syn::Lit::Str(content) = lit {
                                    if namevalue_params.insert(ident, content).is_some() {
                                        Err(SynError::new(
                                            ident.span(),
                                            "this attribute has been set twice",
                                        ))?;
                                    }
                                } else {
                                    Err(SynError::new(
                                        lit.span(),
                                        "this literal should be a string literal",
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
                if word_params.is_empty() && namevalue_params.is_empty() {
                    Err(SynError::new(
                        list.span(),
                        "this attribute should not be empty",
                    ))?;
                }
                match list.ident.to_string().as_ref() {
                    "get" => {
                        let words = check_word_params(&word_params, &[VISIBILITY_OPTIONS])?;
                        let namevalues =
                            check_namevalue_params(&namevalue_params, &[GET_TYPE_OPTIONS])?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(words[0], list.ident.span())?
                        {
                            self.get.vis = choice;
                        }
                        if let Some(choice) =
                            GetTypeConf::parse_from_input(&namevalues, list.ident.span())?
                        {
                            self.get.typ = choice;
                        }
                    }
                    "set" => {
                        let _ = check_namevalue_params(&namevalue_params, &[])?;
                        let words = check_word_params(&word_params, &[VISIBILITY_OPTIONS])?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(words[0], list.ident.span())?
                        {
                            self.set.vis = choice;
                        }
                    }
                    "mut" => {
                        let _ = check_namevalue_params(&namevalue_params, &[])?;
                        let words = check_word_params(&word_params, &[VISIBILITY_OPTIONS])?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(words[0], list.ident.span())?
                        {
                            self.mut_.vis = choice;
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

fn check_word_params<'a>(
    word_params: &::std::collections::HashSet<&syn::Ident>,
    options: &[&[&'a str]],
) -> ParseResult<Vec<Option<&'a str>>> {
    let mut result = vec![None; options.len()];
    let mut find;
    for p in word_params.iter() {
        find = false;
        for (i, group) in options.iter().enumerate() {
            for opt in group.iter() {
                if p == opt {
                    find = true;
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

fn check_namevalue_params<'a>(
    params: &::std::collections::HashMap<&syn::Ident, &syn::LitStr>,
    options: &[(&'a str, Option<&[&'a str]>)],
) -> ParseResult<::std::collections::HashMap<&'a str, String>> {
    let mut result = ::std::collections::HashMap::new();
    let mut find;
    for (n, v) in params.iter() {
        find = false;
        let value = v.value();
        for (k, group_opt) in options.iter() {
            if n == k {
                if let Some(group) = group_opt {
                    for opt in group.iter() {
                        if &value == opt {
                            let _ = result.insert(*k, value.clone());
                            find = true;
                            break;
                        }
                    }
                    if find {
                        break;
                    }
                } else {
                    let _ = result.insert(*k, value);
                    find = true;
                    break;
                }
            }
        }
        if !find {
            Err(SynError::new(n.span(), "this attribute was unknown"))?;
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
                        if list.nested.is_empty() {
                            Err(SynError::new(
                                list.span(),
                                "this attribute should not be empty",
                            ))?;
                        }
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
