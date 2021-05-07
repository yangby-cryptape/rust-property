# Rust-Property

[![License]](#license)
[![GitHub Actions]](https://github.com/yangby-cryptape/rust-property/actions)
[![Crate Badge]](https://crates.io/crates/property)
[![Crate Doc]](https://docs.rs/property)
[![MSRV 1.31.0]][Rust 1.31.0]

Generate several common methods for structs automatically.

[License]: https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-blue.svg
[GitHub Actions]: https://github.com/yangby-cryptape/rust-property/workflows/CI/badge.svg
[Crate Badge]: https://img.shields.io/crates/v/property.svg
[Crate Doc]: https://docs.rs/property/badge.svg
[MSRV 1.31.0]: https://img.shields.io/badge/rust-%3E%3D%201.31.0-blue

## Usage

- Apply the derive proc-macro `#[derive(Property)]` to structs, and use `#[property(..)]` to configure it.

  There are five kinds of configurable attributes: `skip`, `get`, `set`, `mut` and `ord`.

- Set container attributes can change the default settings for all fields.

- Change the settings of a single field via setting field attributes.

- If the `skip` attribute is set, no methods will be generated.

  Don't set `skip` attribute as a container attribute.

- The visibility of a method can be set via `#[property(get(visibility-type))]`

  There are four kinds of the visibility types: `disable`, `public`, `crate` (default for all methods), and `private`.

- The method name can be set in two ways:

  1. Assign a complete name via `#[property(get(name = "method-name"))]`.

  2. Set `prefix` and / or `suffix` via `#[property(set(prefix = "set_"), mut(suffix = "mut_"))]`.

  The default setting for all fields is: `#[property(get(prefix = "", suffix = ""), set(prefix = "set_"), mut(prefix = "mut_"))]`.

- The return type of `get` method can be set via `#[property(get(type = "return-type"))]`.

  There are three kinds of the return types: `ref` (default in most cases), `clone` and `copy`.

- The input type and return type of `set` method can be set via `#[property(set(type = "set-type"))]`.

  There are four kinds of the input types: `ref` (default), `own`, `none` and `replace`:

  - `ref`: input is a mutable reference and return is the mutable reference too.

  - `own`: input is a owned object and return is the owned object too.

  - `none`: input is a mutable reference and no return.

  - `replace`: input is a mutable reference and return the old value.

- If there are more than one filed have the `ord` attribute, the [`PartialEq`] and [`PartialOrd`] will be implemented automatically.

  - A serial number is required for the `ord` field attribute, it's an unsigned number with a `_` prefix.

    The serial numbers could be noncontinuous, but any two number of these could not be equal.

    No serial number is allowed if the `ord` attribute is a container attribute.

  - There are two kind of sort types: `asc` and `desc`.

    The default is ascending (`asc`), it can be changed to descending if the `desc` was set.

[`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
[`PartialOrd`]: https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html

## In Action

### Original Code

```rust
#![no_std]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std as alloc;

use alloc::{string::String, vec::Vec};

use property::Property;

#[derive(Copy, Clone)]
pub enum Species {
    Dog,
    Cat,
    Bird,
    Other,
}

#[derive(Property)]
#[property(get(public), set(private), mut(disable), ord(desc))]
pub struct Pet {
    #[property(get(name = "identification"), set(disable), ord(asc, _2))]
    id: [u8; 32],
    name: String,
    #[property(set(crate, type = "own"), ord(_0))]
    age: u32,
    #[property(get(type = "copy"))]
    species: Species,
    #[property(get(prefix = "is_"), ord(_1))]
    died: bool,
    #[property(get(type = "clone"), set(type = "none"))]
    owner: String,
    family_members: Vec<String>,
    #[property(get(type = "ref"), mut(crate))]
    info: String,
    #[property(get(disable), set(type = "replace"))]
    pub tag: Vec<String>,
    #[property(mut(public, suffix = "_mut"))]
    note: Option<String>,
    #[property(set(type = "replace"))]
    price: Option<u32>,
    #[property(skip)]
    pub reserved: String,
}
```

### Generated Code

```rust
impl Pet {
    #[inline]
    pub fn identification(&self) -> &[u8] {
        &self.id[..]
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.name[..]
    }
    #[inline]
    fn set_name<T: Into<String>>(&mut self, val: T) -> &mut Self {
        self.name = val.into();
        self
    }
    #[inline]
    pub fn age(&self) -> u32 {
        self.age
    }
    #[inline]
    pub(crate) fn set_age<T: Into<u32>>(mut self, val: T) -> Self {
        self.age = val.into();
        self
    }
    #[inline]
    pub fn species(&self) -> Species {
        self.species
    }
    #[inline]
    fn set_species<T: Into<Species>>(&mut self, val: T) -> &mut Self {
        self.species = val.into();
        self
    }
    #[inline]
    pub fn is_died(&self) -> bool {
        self.died
    }
    #[inline]
    fn set_died<T: Into<bool>>(&mut self, val: T) -> &mut Self {
        self.died = val.into();
        self
    }
    #[inline]
    pub fn owner(&self) -> String {
        self.owner.clone()
    }
    #[inline]
    fn set_owner<T: Into<String>>(&mut self, val: T) {
        self.owner = val.into();
    }
    #[inline]
    pub fn family_members(&self) -> &[String] {
        &self.family_members[..]
    }
    #[inline]
    fn set_family_members<T: Into<String>>(
        &mut self,
        val: impl IntoIterator<Item = T>,
    ) -> &mut Self {
        self.family_members = val.into_iter().map(Into::into).collect();
        self
    }
    #[inline]
    pub fn info(&self) -> &String {
        &self.info
    }
    #[inline]
    fn set_info<T: Into<String>>(&mut self, val: T) -> &mut Self {
        self.info = val.into();
        self
    }
    #[inline]
    pub(crate) fn mut_info(&mut self) -> &mut String {
        &mut self.info
    }
    #[inline]
    fn set_tag<T: Into<String>>(&mut self, val: impl IntoIterator<Item = T>) -> Vec<String> {
        ::core::mem::replace(&mut self.tag, val.into_iter().map(Into::into).collect())
    }
    #[inline]
    pub fn note(&self) -> Option<&String> {
        self.note.as_ref()
    }
    #[inline]
    fn set_note<T: Into<Option<String>>>(&mut self, val: T) -> &mut Self {
        self.note = val.into();
        self
    }
    #[inline]
    pub fn note_mut(&mut self) -> &mut Option<String> {
        &mut self.note
    }
    #[inline]
    pub fn price(&self) -> Option<u32> {
        self.price
    }
    #[inline]
    fn set_price<T: Into<Option<u32>>>(&mut self, val: T) -> Option<u32> {
        ::core::mem::replace(&mut self.price, val.into())
    }
}
impl PartialEq for Pet {
    fn eq(&self, other: &Self) -> bool {
        self.age == other.age && self.died == other.died && self.id == other.id
    }
}
impl PartialOrd for Pet {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        let result = other.age.partial_cmp(&self.age);
        if result != Some(::core::cmp::Ordering::Equal) {
            return result;
        }
        let result = other.died.partial_cmp(&self.died);
        if result != Some(::core::cmp::Ordering::Equal) {
            return result;
        }
        let result = self.id.partial_cmp(&other.id);
        if result != Some(::core::cmp::Ordering::Equal) {
            return result;
        }
        Some(::core::cmp::Ordering::Equal)
    }
}
```

Enjoy it!

## Minimum Supported Rust Version

[Rust 1.31.0].

## License

Licensed under either of [Apache License, Version 2.0] or [MIT License], at your option.

[Apache License, Version 2.0]: LICENSE-APACHE
[MIT License]: LICENSE-MIT
[Rust 1.31.0]: https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html
