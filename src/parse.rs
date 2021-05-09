// Copyright (C) 2019-2021 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Once,
};

use quote::quote;
use syn::{parse::Result as ParseResult, spanned::Spanned, Error as SynError};

const ATTR_NAME: &str = "property";
const SKIP: &str = "skip";
const GET_TYPE_OPTIONS: (&str, Option<&[&str]>) = ("type", Some(&["auto", "ref", "copy", "clone"]));
const SET_TYPE_OPTIONS: (&str, Option<&[&str]>) =
    ("type", Some(&["ref", "own", "none", "replace"]));
const NAME_OPTION: (&str, Option<&[&str]>) = ("name", None);
const STRIP_OPTION: &[&str] = &["strip_option"];
const PREFIX_OPTION: (&str, Option<&[&str]>) = ("prefix", None);
const SUFFIX_OPTION: (&str, Option<&[&str]>) = ("suffix", None);
const VISIBILITY_OPTIONS: &[&str] = &["disable", "public", "crate", "private"];
const SORT_TYPE_OPTIONS: &[&str] = &["asc", "desc"];

static INIT_DEFAULT: Once = Once::new();
static mut CRATE_CONF: Option<FieldConf> = None;
static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PropertyType {
    Crate,
    Container,
    Field,
}

pub(crate) struct CrateConfDef {
    pub(crate) conf: FieldConf,
}

pub(crate) struct ContainerDef {
    pub(crate) name: syn::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) fields: Vec<FieldDef>,
}

pub(crate) struct FieldDef {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
    pub(crate) conf: FieldConf,
}

#[derive(Clone, Copy)]
pub(crate) enum GetTypeConf {
    Auto,
    Ref,
    Copy_,
    Clone_,
}

#[derive(Clone, Copy)]
pub(crate) enum SetTypeConf {
    Ref,
    Own,
    None_,
    Replace,
}

#[derive(Clone, Copy)]
pub(crate) enum VisibilityConf {
    Disable,
    Public,
    Crate,
    Private,
}

#[derive(Clone, Copy)]
pub(crate) enum SortTypeConf {
    Ascending,
    Descending,
}

#[derive(Clone)]
pub(crate) enum MethodNameConf {
    Name(String),
    Format { prefix: String, suffix: String },
}

#[derive(Clone)]
pub(crate) struct GetFieldConf {
    pub(crate) vis: VisibilityConf,
    pub(crate) name: MethodNameConf,
    pub(crate) typ: GetTypeConf,
}

#[derive(Clone)]
pub(crate) struct SetFieldConf {
    pub(crate) vis: VisibilityConf,
    pub(crate) name: MethodNameConf,
    pub(crate) typ: SetTypeConf,
    pub(crate) strip_option: bool,
}

#[derive(Clone)]
pub(crate) struct MutFieldConf {
    pub(crate) vis: VisibilityConf,
    pub(crate) name: MethodNameConf,
}

#[derive(Clone)]
pub(crate) struct OrdFieldConf {
    pub(crate) number: Option<usize>,
    pub(crate) sort_type: SortTypeConf,
}

#[derive(Clone)]
pub(crate) struct FieldConf {
    pub(crate) get: GetFieldConf,
    pub(crate) set: SetFieldConf,
    pub(crate) mut_: MutFieldConf,
    pub(crate) ord: OrdFieldConf,
    pub(crate) skip: bool,
}

impl syn::parse::Parse for CrateConfDef {
    fn parse(input: syn::parse::ParseStream) -> ParseResult<Self> {
        let attr_args =
            syn::punctuated::Punctuated::<syn::NestedMeta, syn::Token![,]>::parse_terminated(
                input,
            )?;
        let mut conf = FieldConf::default();
        for nested_meta in attr_args.iter() {
            parse_nested_meta(&mut conf, nested_meta, PropertyType::Crate)?;
        }
        Ok(Self { conf })
    }
}

impl CrateConfDef {
    pub(crate) fn set_default_conf(self) {
        let Self { conf } = self;
        let call_count = CALL_COUNT.load(Ordering::SeqCst);
        unsafe {
            if CRATE_CONF.is_some() {
                panic!(
                    "The default property for the whole crate should be \
                     set only once for each crate."
                );
            } else if call_count > 0 {
                panic!(
                    "Some properties of containers or fields was set \
                     before the default property for the whole crate has been taken effect."
                );
            }
            INIT_DEFAULT.call_once(|| {
                CRATE_CONF = Some(conf);
            });
        }
    }

    fn get_default_conf() -> FieldConf {
        let _ = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        unsafe { CRATE_CONF.as_ref().map(ToOwned::to_owned) }.unwrap_or_else(Default::default)
    }
}

impl syn::parse::Parse for ContainerDef {
    fn parse(input: syn::parse::ParseStream) -> ParseResult<Self> {
        let derive_input: syn::DeriveInput = input.parse()?;
        let attrs_span = derive_input.span();
        let syn::DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = derive_input;
        let ident_span = ident.span();
        match data {
            syn::Data::Struct(data) => match data.fields {
                syn::Fields::Named(named_fields) => {
                    let conf = ContainerDef::parse_attrs(
                        attrs_span,
                        CrateConfDef::get_default_conf(),
                        &attrs[..],
                    )?;
                    Ok(Self {
                        name: ident,
                        generics,
                        fields: FieldDef::parse_named_fields(named_fields, conf, ident_span)?,
                    })
                }
                _ => Err(SynError::new(ident_span, "only support named fields")),
            },
            _ => Err(SynError::new(ident_span, "only support structs")),
        }
    }
}

impl ContainerDef {
    fn parse_attrs(
        span: proc_macro2::Span,
        conf: FieldConf,
        attrs: &[syn::Attribute],
    ) -> ParseResult<FieldConf> {
        parse_attrs(span, conf, attrs, PropertyType::Container)
    }
}

impl FieldDef {
    fn parse_named_fields(
        named_fields: syn::FieldsNamed,
        conf: FieldConf,
        span: proc_macro2::Span,
    ) -> ParseResult<Vec<Self>> {
        let mut fields = Vec::new();
        for f in named_fields.named.into_iter() {
            let syn::Field {
                attrs, ident, ty, ..
            } = f.clone();
            let conf = FieldDef::parse_attrs(f.span(), conf.clone(), &attrs[..])?;
            let ident = ident.ok_or_else(|| SynError::new(f.span(), "unreachable"))?;
            let field = Self { ident, ty, conf };
            fields.push(field);
        }
        if fields.is_empty() {
            Err(SynError::new(span, "nothing can do for an empty struct"))
        } else {
            Ok(fields)
        }
    }

    fn parse_attrs(
        span: proc_macro2::Span,
        conf: FieldConf,
        attrs: &[syn::Attribute],
    ) -> ParseResult<FieldConf> {
        parse_attrs(span, conf, attrs, PropertyType::Field)
    }
}

impl GetTypeConf {
    pub(crate) fn parse_from_input(
        namevalue_params: &::std::collections::HashMap<&str, String>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let choice = match namevalue_params.get("type").map(AsRef::as_ref) {
            None => None,
            Some("auto") => Some(GetTypeConf::Auto),
            Some("ref") => Some(GetTypeConf::Ref),
            Some("copy") => Some(GetTypeConf::Copy_),
            Some("clone") => Some(GetTypeConf::Clone_),
            _ => return Err(SynError::new(span, "unreachable result")),
        };
        Ok(choice)
    }
}

impl SetTypeConf {
    pub(crate) fn parse_from_input(
        namevalue_params: &::std::collections::HashMap<&str, String>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let choice = match namevalue_params.get("type").map(AsRef::as_ref) {
            None => None,
            Some("ref") => Some(SetTypeConf::Ref),
            Some("own") => Some(SetTypeConf::Own),
            Some("none") => Some(SetTypeConf::None_),
            Some("replace") => Some(SetTypeConf::Replace),
            _ => return Err(SynError::new(span, "unreachable result")),
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
            _ => return Err(SynError::new(span, "unreachable result")),
        };
        Ok(choice)
    }

    pub(crate) fn to_ts(self) -> Option<proc_macro2::TokenStream> {
        match self {
            VisibilityConf::Disable => None,
            VisibilityConf::Public => Some(quote!(pub)),
            VisibilityConf::Crate => Some(quote!(pub(crate))),
            VisibilityConf::Private => Some(quote!()),
        }
    }
}

impl SortTypeConf {
    pub(crate) fn parse_from_input(
        input: Option<&str>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let choice = match input {
            None => None,
            Some("asc") => Some(SortTypeConf::Ascending),
            Some("desc") => Some(SortTypeConf::Descending),
            _ => return Err(SynError::new(span, "unreachable result")),
        };
        Ok(choice)
    }

    pub(crate) fn is_ascending(self) -> bool {
        match self {
            SortTypeConf::Ascending => true,
            SortTypeConf::Descending => false,
        }
    }
}

impl MethodNameConf {
    pub(crate) fn parse_from_input(
        namevalue_params: &::std::collections::HashMap<&str, String>,
        span: proc_macro2::Span,
    ) -> ParseResult<Option<Self>> {
        let name_opt = namevalue_params.get("name").map(ToOwned::to_owned);
        let prefix_opt = namevalue_params.get("prefix").map(ToOwned::to_owned);
        let suffix_opt = namevalue_params.get("suffix").map(ToOwned::to_owned);
        if let Some(name) = name_opt {
            if prefix_opt.is_some() || suffix_opt.is_some() {
                Err(SynError::new(
                    span,
                    "do not set prefix or suffix if name was set",
                ))
            } else {
                Ok(Some(MethodNameConf::Name(name)))
            }
        } else {
            let choice = match (prefix_opt, suffix_opt) {
                (Some(prefix), Some(suffix)) => Some(MethodNameConf::Format { prefix, suffix }),
                (Some(prefix), None) => Some(MethodNameConf::Format {
                    prefix,
                    suffix: "".to_owned(),
                }),
                (None, Some(suffix)) => Some(MethodNameConf::Format {
                    prefix: "".to_owned(),
                    suffix,
                }),
                (None, None) => None,
            };
            Ok(choice)
        }
    }

    pub(crate) fn complete(&self, field_name: &syn::Ident) -> syn::Ident {
        let method_name = match self {
            MethodNameConf::Name(ref name) => name.to_owned(),
            MethodNameConf::Format { prefix, suffix } => {
                format!("{}{}{}", prefix, field_name.to_string(), suffix)
            }
        };
        syn::Ident::new(&method_name, field_name.span())
    }
}

impl OrdFieldConf {
    pub(crate) fn parse_from_path_params<'a>(
        path_params: &::std::collections::HashSet<&syn::Path>,
        options: &[&'a str],
        span: proc_macro2::Span,
        prop_type: PropertyType,
    ) -> ParseResult<(Option<&'a str>, Option<usize>)> {
        let mut sort_type = None;
        let mut number_opt = None;
        for p in path_params.iter() {
            let s = p
                .get_ident()
                .ok_or_else(|| SynError::new(p.span(), "this attribute should be a single ident"))?
                .to_string();
            if options.iter().any(|opt| *opt == s.as_str()) {
                if sort_type.is_some() {
                    return Err(SynError::new(
                        p.span(),
                        "this kind of attribute has been set twice",
                    ));
                }
                for opt in options.iter() {
                    if *opt == s.as_str() {
                        sort_type = Some(*opt);
                        break;
                    }
                }
            } else if &s.as_bytes()[..1] == b"_" {
                if prop_type != PropertyType::Field {
                    return Err(SynError::new(
                        p.span(),
                        "the serial number could not be set as a crate or container attribute",
                    ));
                } else if let Ok(n) = s.as_str()[1..].parse::<usize>() {
                    if number_opt.is_some() {
                        return Err(SynError::new(
                            p.span(),
                            "the serial number has been set twice",
                        ));
                    }
                    number_opt = Some(n);
                } else {
                    return Err(SynError::new(
                        p.span(),
                        "the serial number should be an unsigned number with a `_` prefix",
                    ));
                }
            } else {
                return Err(SynError::new(p.span(), "this attribute was unknown"));
            }
        }
        if prop_type == PropertyType::Field && number_opt.is_none() {
            Err(SynError::new(span, "no serial number was set"))
        } else {
            Ok((sort_type, number_opt))
        }
    }
}

impl ::std::default::Default for FieldConf {
    fn default() -> Self {
        Self {
            get: GetFieldConf {
                vis: VisibilityConf::Crate,
                name: MethodNameConf::Format {
                    prefix: "".to_owned(),
                    suffix: "".to_owned(),
                },
                typ: GetTypeConf::Auto,
            },
            set: SetFieldConf {
                vis: VisibilityConf::Crate,
                name: MethodNameConf::Format {
                    prefix: "set_".to_owned(),
                    suffix: "".to_owned(),
                },
                typ: SetTypeConf::Ref,
                strip_option: false,
            },
            mut_: MutFieldConf {
                vis: VisibilityConf::Crate,
                name: MethodNameConf::Format {
                    prefix: "mut_".to_owned(),
                    suffix: "".to_owned(),
                },
            },
            ord: OrdFieldConf {
                number: None,
                sort_type: SortTypeConf::Ascending,
            },
            skip: false,
        }
    }
}

impl FieldConf {
    fn apply_attrs(&mut self, meta: &syn::Meta, prop_type: PropertyType) -> ParseResult<()> {
        match meta {
            syn::Meta::Path(path) => {
                if path.is_ident(SKIP) {
                    self.skip = true;
                } else {
                    return Err(SynError::new(path.span(), "this attribute was unknown"));
                }
            }
            syn::Meta::List(list) => {
                let mut path_params = ::std::collections::HashSet::new();
                let mut namevalue_params = ::std::collections::HashMap::new();
                for nested_meta in list.nested.iter() {
                    match nested_meta {
                        syn::NestedMeta::Meta(meta) => match meta {
                            syn::Meta::Path(path) => {
                                if !path_params.insert(path) {
                                    return Err(SynError::new(
                                        path.span(),
                                        "this attribute has been set twice",
                                    ));
                                }
                            }
                            syn::Meta::NameValue(mnv) => {
                                let syn::MetaNameValue { path, lit, .. } = mnv;
                                if let syn::Lit::Str(content) = lit {
                                    if namevalue_params.insert(path, content).is_some() {
                                        return Err(SynError::new(
                                            path.span(),
                                            "this attribute has been set twice",
                                        ));
                                    }
                                } else {
                                    return Err(SynError::new(
                                        lit.span(),
                                        "this literal should be a string literal",
                                    ));
                                }
                            }
                            _ => {
                                return Err(SynError::new(
                                    meta.span(),
                                    "this attribute should be a path or a name-value pair",
                                ));
                            }
                        },
                        syn::NestedMeta::Lit(lit) => {
                            return Err(SynError::new(
                                lit.span(),
                                "this attribute should not be a literal",
                            ));
                        }
                    }
                }
                if path_params.is_empty() && namevalue_params.is_empty() {
                    return Err(SynError::new(
                        list.span(),
                        "this attribute should not be empty",
                    ));
                }
                match list
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        SynError::new(list.path.span(), "this attribute should be a single ident")
                    })?
                    .to_string()
                    .as_ref()
                {
                    "get" => {
                        let paths = check_path_params(&path_params, &[VISIBILITY_OPTIONS])?;
                        let namevalues = check_namevalue_params(
                            &namevalue_params,
                            &[NAME_OPTION, PREFIX_OPTION, SUFFIX_OPTION, GET_TYPE_OPTIONS],
                        )?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(paths[0], list.path.span())?
                        {
                            self.get.vis = choice;
                        }
                        if let Some(choice) =
                            MethodNameConf::parse_from_input(&namevalues, list.path.span())?
                        {
                            self.get.name = choice;
                        }
                        if let Some(choice) =
                            GetTypeConf::parse_from_input(&namevalues, list.path.span())?
                        {
                            self.get.typ = choice;
                        }
                    }
                    "set" => {
                        let paths = check_path_params(&path_params, &[VISIBILITY_OPTIONS, STRIP_OPTION])?;
                        let namevalues = check_namevalue_params(
                            &namevalue_params,
                            &[NAME_OPTION, PREFIX_OPTION, SUFFIX_OPTION, SET_TYPE_OPTIONS],
                        )?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(paths[0], list.path.span())?
                        {
                            self.set.vis = choice;
                        }
                        if let Some(choice) =
                            MethodNameConf::parse_from_input(&namevalues, list.path.span())?
                        {
                            self.set.name = choice;
                        }
                        if let Some(choice) =
                            SetTypeConf::parse_from_input(&namevalues, list.path.span())?
                        {
                            self.set.typ = choice;
                        }
                        self.set.strip_option = paths[1].is_some();
                    }
                    "mut" => {
                        let paths = check_path_params(&path_params, &[VISIBILITY_OPTIONS])?;
                        let namevalues = check_namevalue_params(
                            &namevalue_params,
                            &[NAME_OPTION, PREFIX_OPTION, SUFFIX_OPTION],
                        )?;
                        if let Some(choice) =
                            VisibilityConf::parse_from_input(paths[0], list.path.span())?
                        {
                            self.mut_.vis = choice;
                        }
                        if let Some(choice) =
                            MethodNameConf::parse_from_input(&namevalues, list.path.span())?
                        {
                            self.mut_.name = choice;
                        }
                    }
                    "ord" => {
                        let (sort_type_opt, number_opt) = OrdFieldConf::parse_from_path_params(
                            &path_params,
                            SORT_TYPE_OPTIONS,
                            list.path.span(),
                            prop_type,
                        )?;
                        if let Some(choice) =
                            SortTypeConf::parse_from_input(sort_type_opt, list.path.span())?
                        {
                            self.ord.sort_type = choice;
                        }
                        self.ord.number = number_opt;
                    }
                    attr => {
                        return Err(SynError::new(
                            list.path.span(),
                            format!("unsupport attribute `{}`", attr),
                        ));
                    }
                }
            }
            syn::Meta::NameValue(name_value) => {
                return Err(SynError::new(
                    name_value.span(),
                    "this attribute should not be a name-value pair",
                ));
            }
        }
        Ok(())
    }
}

fn check_path_params<'a>(
    path_params: &::std::collections::HashSet<&syn::Path>,
    options: &[&[&'a str]],
) -> ParseResult<Vec<Option<&'a str>>> {
    let mut result = vec![None; options.len()];
    let mut find;
    for p in path_params.iter() {
        find = false;
        for (i, group) in options.iter().enumerate() {
            for opt in group.iter() {
                if p.is_ident(opt) {
                    find = true;
                    if result[i].is_some() {
                        return Err(SynError::new(
                            p.span(),
                            "this kind of attribute has been set twice",
                        ));
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
            return Err(SynError::new(p.span(), "this attribute was unknown"));
        }
    }
    Ok(result)
}

fn check_namevalue_params<'a>(
    params: &::std::collections::HashMap<&syn::Path, &syn::LitStr>,
    options: &[(&'a str, Option<&[&'a str]>)],
) -> ParseResult<::std::collections::HashMap<&'a str, String>> {
    let mut result = ::std::collections::HashMap::new();
    let mut find;
    for (n, v) in params.iter() {
        find = false;
        let value = v.value();
        for (k, group_opt) in options.iter() {
            if n.is_ident(k) {
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
            return Err(SynError::new(n.span(), "this attribute was unknown"));
        }
    }
    Ok(result)
}

fn parse_attrs(
    span: proc_macro2::Span,
    mut conf: FieldConf,
    attrs: &[syn::Attribute],
    prop_type: PropertyType,
) -> ParseResult<FieldConf> {
    for attr in attrs.iter() {
        if let syn::AttrStyle::Outer = attr.style {
            let meta = attr
                .parse_meta()
                .map_err(|_| SynError::new(span, "failed to parse the attributes"))?;
            match meta {
                syn::Meta::Path(path) => {
                    if path.is_ident(ATTR_NAME) {
                        return Err(SynError::new(
                            path.span(),
                            "the attribute should not be a path",
                        ));
                    }
                }
                syn::Meta::List(list) => {
                    if list.path.is_ident(ATTR_NAME) {
                        if list.nested.is_empty() {
                            return Err(SynError::new(
                                list.span(),
                                "this attribute should not be empty",
                            ));
                        }
                        for nested_meta in list.nested.iter() {
                            parse_nested_meta(&mut conf, nested_meta, prop_type)?;
                        }
                    }
                }
                syn::Meta::NameValue(name_value) => {
                    if name_value.path.is_ident(ATTR_NAME) {
                        return Err(SynError::new(
                            name_value.span(),
                            "the attribute should not be a name-value pair",
                        ));
                    }
                }
            }
        }
    }
    Ok(conf)
}

fn parse_nested_meta(
    conf: &mut FieldConf,
    nested_meta: &syn::NestedMeta,
    prop_type: PropertyType,
) -> ParseResult<()> {
    match nested_meta {
        syn::NestedMeta::Meta(meta) => {
            conf.apply_attrs(meta, prop_type)?;
            Ok(())
        }
        syn::NestedMeta::Lit(lit) => Err(SynError::new(
            lit.span(),
            "the attribute in nested meta should not be a literal",
        )),
    }
}
