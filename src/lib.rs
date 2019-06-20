// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate proc_macro;

use quote::quote;

mod generate;
mod parse;

use crate::{
    generate::{FieldType, GetType},
    parse::{FieldDef, GetTypeConf, PropertyDef, SetTypeConf},
};

/// Generate several common methods for structs automatically.
#[proc_macro_derive(Property, attributes(property))]
pub fn derive_property(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as PropertyDef);
    let expanded = {
        let PropertyDef {
            name,
            generics,
            fields,
        } = input;
        let (impl_generics, type_generics, where_clause_opt) = generics.split_for_impl();
        let methods = fields.into_iter().fold(Vec::new(), |mut r, f| {
            r.append(&mut derive_property_for_field(f));
            r
        });
        quote!(
            impl #impl_generics #name #type_generics #where_clause_opt {
                #(#[inline(always)] #methods)*
            }
        )
    };
    expanded.into()
}

fn derive_property_for_field(field: FieldDef) -> Vec<proc_macro2::TokenStream> {
    let mut property = Vec::new();
    let field_type = &field.ty;
    let field_name = &field.ident;
    let field_conf = &field.conf;
    let prop_field_type = FieldType::from_type(field_type);
    if let Some(ts) = field_conf.get.vis.to_ts().and_then(|visibility| {
        let method_name = field_conf.get.name.complete(field_name);
        let get_type = match field_conf.get.typ {
            GetTypeConf::NotSet => GetType::from_field_type(&prop_field_type),
            GetTypeConf::Ref => GetType::Ref,
            GetTypeConf::Copy_ => GetType::Copy_,
            GetTypeConf::Clone_ => GetType::Clone_,
        };
        let generated = match get_type {
            GetType::Ref => quote!(
                #visibility fn #method_name(&self) -> &#field_type {
                    &self.#field_name
                }
            ),
            GetType::Copy_ => quote!(
                #visibility fn #method_name(&self) -> #field_type {
                    self.#field_name
                }
            ),
            GetType::Clone_ => quote!(
                #visibility fn #method_name(&self) -> #field_type {
                    self.#field_name.clone()
                }
            ),
            GetType::String_ => quote!(
                #visibility fn #method_name(&self) -> &str {
                    &self.#field_name[..]
                }
            ),
            GetType::Slice(field_type) => quote!(
                #visibility fn #method_name(&self) -> &#field_type {
                    &self.#field_name[..]
                }
            ),
            GetType::Option_(field_type) => quote!(
                #visibility fn #method_name(&self) -> Option<&#field_type> {
                    self.#field_name.as_ref()
                }
            ),
        };
        Some(generated)
    }) {
        property.push(ts);
    }
    if let Some(ts) = field_conf.set.vis.to_ts().and_then(|visibility| {
        let method_name = field_conf.set.name.complete(field_name);
        let generated = match prop_field_type {
            FieldType::Vector(inner_type) => match field_conf.set.typ {
                SetTypeConf::Ref => quote!(
                    #visibility fn #method_name<T: Into<#inner_type>>(
                       &mut self,
                       val: impl IntoIterator<Item = T>
                    ) -> &mut Self {
                        self.#field_name = val.into_iter().map(Into::into).collect();
                        self
                    }
                ),
                SetTypeConf::Own => quote!(
                    #visibility fn #method_name<T: Into<#inner_type>>(
                        mut self,
                        val: impl IntoIterator<Item = T>
                    ) -> Self {
                        self.#field_name = val.into_iter().map(Into::into).collect();
                        self
                    }
                ),
            },
            _ => match field_conf.set.typ {
                SetTypeConf::Ref => quote!(
                    #visibility fn #method_name<T: Into<#field_type>>(
                        &mut self, val: T
                    ) -> &mut Self {
                        self.#field_name = val.into();
                        self
                    }
                ),
                SetTypeConf::Own => quote!(
                    #visibility fn #method_name<T: Into<#field_type>>(
                        mut self, val: T
                    ) -> Self {
                        self.#field_name = val.into();
                        self
                    }
                ),
            },
        };
        Some(generated)
    }) {
        property.push(ts);
    }
    if let Some(ts) = field_conf.mut_.vis.to_ts().and_then(|visibility| {
        let method_name = field_conf.mut_.name.complete(field_name);
        let generated = quote!(
            #visibility fn #method_name(&mut self) -> &mut #field_type {
                &mut self.#field_name
            }
        );
        Some(generated)
    }) {
        property.push(ts);
    }
    property
}
