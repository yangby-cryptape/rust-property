// Copyright (C) 2019-2020 Boyu Yang
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
    let property = syn::parse_macro_input!(input as PropertyDef);
    let expanded = {
        let name = &property.name;
        let (impl_generics, type_generics, where_clause_opt) = property.generics.split_for_impl();
        let methods = property.fields.iter().fold(Vec::new(), |mut r, f| {
            if !f.conf.skip {
                r.append(&mut derive_property_for_field(f));
            }
            r
        });
        let impl_methods = quote!(
            impl #impl_generics #name #type_generics #where_clause_opt {
                #(#[inline] #methods)*
            }
        );
        if let Some(impl_traits) = implement_traits(&property) {
            quote!(#impl_methods #impl_traits)
        } else {
            impl_methods
        }
    };
    expanded.into()
}

fn implement_traits(property: &PropertyDef) -> Option<proc_macro2::TokenStream> {
    let name = &property.name;
    let mut ordered: Vec<_> = property
        .fields
        .iter()
        .filter(|f| f.conf.ord.number.is_some())
        .collect();
    if ordered.is_empty() {
        None
    } else {
        ordered.sort_by(|f1, f2| {
            let n1 = f1.conf.ord.number.unwrap();
            let n2 = f2.conf.ord.number.unwrap();
            n1.cmp(&n2)
        });
        let has_same_serial_number = ordered.windows(2).any(|f| {
            let n1 = f[0].conf.ord.number.unwrap();
            let n2 = f[1].conf.ord.number.unwrap();
            n1 == n2
        });
        if has_same_serial_number {
            panic!("there are at least two fields that have same serial number");
        }
        let partial_eq_stmt = ordered.iter().fold(Vec::new(), |mut r, f| {
            if !r.is_empty() {
                r.push(quote!(&&));
            }
            let field_name = &f.ident;
            r.push(quote!(self.#field_name == other.#field_name));
            r
        });
        let partial_ord_stmt = ordered.iter().fold(Vec::new(), |mut r, f| {
            let field_name = &f.ident;
            r.push(if f.conf.ord.sort_type.is_ascending() {
                quote!(let result = self.#field_name.partial_cmp(&other.#field_name);)
            } else {
                quote!(let result = other.#field_name.partial_cmp(&self.#field_name);)
            });
            r.push(quote!(if result != Some(::core::cmp::Ordering::Equal) {
                return result;
            }));
            r
        });
        let stmts = quote!(
            impl PartialEq for #name {
                fn eq(&self, other: &Self) -> bool {
                    #(#partial_eq_stmt)*
                }
            }

            impl PartialOrd for #name {
                fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                    #(#partial_ord_stmt)*
                    Some(::core::cmp::Ordering::Equal)
                }
            }
        );
        Some(stmts)
    }
}

fn derive_property_for_field(field: &FieldDef) -> Vec<proc_macro2::TokenStream> {
    let mut property = Vec::new();
    let field_type = &field.ty;
    let field_name = &field.ident;
    let field_conf = &field.conf;
    let prop_field_type = FieldType::from_type(field_type);
    if let Some(ts) = field_conf.get.vis.to_ts().map(|visibility| {
        let method_name = field_conf.get.name.complete(field_name);
        let get_type = match field_conf.get.typ {
            GetTypeConf::NotSet => GetType::from_field_type(&prop_field_type),
            GetTypeConf::Ref => GetType::Ref,
            GetTypeConf::Copy_ => GetType::Copy_,
            GetTypeConf::Clone_ => GetType::Clone_,
        };
        match get_type {
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
        }
    }) {
        property.push(ts);
    }
    if let Some(ts) = field_conf.set.vis.to_ts().map(|visibility| {
        let method_name = field_conf.set.name.complete(field_name);
        match prop_field_type {
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
                SetTypeConf::None_ => quote!(
                    #visibility fn #method_name<T: Into<#inner_type>>(
                       &mut self,
                       val: impl IntoIterator<Item = T>
                    ) {
                        self.#field_name = val.into_iter().map(Into::into).collect();
                    }
                ),
                SetTypeConf::Replace => quote!(
                    #visibility fn #method_name<T: Into<#inner_type>>(
                       &mut self,
                       val: impl IntoIterator<Item = T>
                    ) -> #field_type {
                        ::core::mem::replace(&mut self.#field_name, val.into_iter().map(Into::into).collect())
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
                SetTypeConf::None_ => quote!(
                    #visibility fn #method_name<T: Into<#field_type>>(
                        &mut self, val: T
                    ) {
                        self.#field_name = val.into();
                    }
                ),
                SetTypeConf::Replace => quote!(
                    #visibility fn #method_name<T: Into<#field_type>>(
                        &mut self, val: T
                    ) -> #field_type {
                        ::core::mem::replace(&mut self.#field_name, val.into())
                    }
                ),
            },
        }
    }) {
        property.push(ts);
    }
    if let Some(ts) = field_conf.mut_.vis.to_ts().map(|visibility| {
        let method_name = field_conf.mut_.name.complete(field_name);
        quote!(
            #visibility fn #method_name(&mut self) -> &mut #field_type {
                &mut self.#field_name
            }
        )
    }) {
        property.push(ts);
    }
    property
}
