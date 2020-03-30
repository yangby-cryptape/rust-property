// Copyright (C) 2019-2020 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use quote::quote;

pub(crate) enum GetType {
    Ref,
    Copy_,
    Clone_,
    String_,
    Slice(syn::TypeSlice),
    Option_(proc_macro2::TokenStream),
}

pub(crate) enum FieldType {
    Number,
    Boolean,
    Character,
    String_,
    Array(syn::TypeArray),
    Vector(syn::Type),
    Option_(proc_macro2::TokenStream),
    Unhandled,
}

impl GetType {
    pub(crate) fn from_field_type(ty: &FieldType) -> Self {
        match ty {
            FieldType::Number | FieldType::Boolean | FieldType::Character => Self::Copy_,
            FieldType::String_ => Self::String_,
            FieldType::Array(type_array) => {
                let syn::TypeArray {
                    bracket_token,
                    elem,
                    ..
                } = type_array.clone();
                Self::Slice(syn::TypeSlice {
                    bracket_token,
                    elem,
                })
            }
            FieldType::Vector(inner_type) => Self::Slice(syn::TypeSlice {
                bracket_token: syn::token::Bracket::default(),
                elem: Box::new(inner_type.clone()),
            }),
            FieldType::Option_(inner_type) => Self::Option_(inner_type.clone()),
            FieldType::Unhandled => Self::Ref,
        }
    }
}

impl FieldType {
    pub(crate) fn from_type(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Path(type_path) => {
                let segs = &type_path.path.segments;
                if segs.len() == 1 {
                    match segs[0].ident.to_string().as_ref() {
                        "f32" | "f64" => Self::Number,
                        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => Self::Number,
                        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => Self::Number,
                        "bool" => Self::Boolean,
                        "char" => Self::Character,
                        "String" => Self::String_,
                        "Vec" => {
                            if let syn::PathArguments::AngleBracketed(inner) =
                                &type_path.path.segments[0].arguments
                            {
                                if let syn::GenericArgument::Type(ref inner_type) = inner.args[0] {
                                    Self::Vector(inner_type.clone())
                                } else {
                                    unreachable!()
                                }
                            } else {
                                unreachable!()
                            }
                        }
                        "Option" => {
                            if let syn::PathArguments::AngleBracketed(inner) =
                                &type_path.path.segments[0].arguments
                            {
                                let args = &inner.args;
                                Self::Option_(quote!(#args))
                            } else {
                                unreachable!()
                            }
                        }
                        _ => Self::Unhandled,
                    }
                } else {
                    Self::Unhandled
                }
            }
            syn::Type::Array(type_array) => Self::Array(type_array.clone()),
            _ => Self::Unhandled,
        }
    }
}
