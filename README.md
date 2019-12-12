# Rust-Property

[![License]](#license)
[![Travis CI]](https://travis-ci.com/yangby-cryptape/rust-property)
[![Crate Badge]](https://crates.io/crates/property)
[![Crate Doc]](https://docs.rs/property)

Generate several common methods for structs automatically.

[License]: https://img.shields.io/badge/License-Apache--2.0%20OR%20MIT-blue.svg
[Travis CI]: https://img.shields.io/travis/com/yangby-cryptape/rust-property.svg
[Crate Badge]: https://img.shields.io/crates/v/property.svg
[Crate Doc]: https://docs.rs/property/badge.svg

## Usage

- Apply the derive proc-macro `#[derive(Property)]` to structs, and use `#[property(..)]` to configure it.

  There are four kinds of configurable attributes: `get`, `set`, `mut` and `ord`.

- Set container attributes can change the default settings for all fields.

- Change the settings of a single field via setting field attributes.

- The visibility of a method can be set via `#[property(get(visibility-type))]`

  There are four kinds of the visibility types: `disable`, `public`, `crate` (default for all methods), and `private`.

- The method name can be set in two ways:

  1. Assign a complete name via `#[property(get(name = "method-name"))]`.

  2. Set `prefix` and / or `suffix` via `#[property(set(prefix = "set_"), mut(suffix = "mut_"))]`.

  The default setting for all fields is: `#[property(get(prefix = "", suffix = ""), set(prefix = "set_"), mut(prefix = "mut_"))]`.

- The return type of `get` method can be set via `#[property(get(type = "return-type"))]`.

  There are three kinds of the return types: `ref` (default in most cases), `clone` and `copy`.

- The input type of `set` method can be set via `#[property(set(type = "input-type"))]`.

  There are two kinds of the input types: `ref` (default) and `own`.

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
    #[property(get(type = "clone"))]
    owner: String,
    family_members: Vec<String>,
    #[property(get(type = "ref"), mut(crate))]
    info: String,
    #[property(mut(public, suffix = "_mut"))]
    note: Option<String>,
}
```

### Generated Code

```rust
impl Pet {
    #[inline(always)]
    pub fn identification(&self) -> &[u8] {
        &self.id[..]
    }
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name[..]
    }
    #[inline(always)]
    fn set_name<T: Into<String>>(&mut self, val: T) -> &mut Self {
        self.name = val.into();
        self
    }
    #[inline(always)]
    pub fn age(&self) -> u32 {
        self.age
    }
    #[inline(always)]
    pub(crate) fn set_age<T: Into<u32>>(mut self, val: T) -> Self {
        self.age = val.into();
        self
    }
    #[inline(always)]
    pub fn species(&self) -> Species {
        self.species
    }
    #[inline(always)]
    fn set_species<T: Into<Species>>(&mut self, val: T) -> &mut Self {
        self.species = val.into();
        self
    }
    #[inline(always)]
    pub fn is_died(&self) -> bool {
        self.died
    }
    #[inline(always)]
    fn set_died<T: Into<bool>>(&mut self, val: T) -> &mut Self {
        self.died = val.into();
        self
    }
    #[inline(always)]
    pub fn owner(&self) -> String {
        self.owner.clone()
    }
    #[inline(always)]
    fn set_owner<T: Into<String>>(&mut self, val: T) -> &mut Self {
        self.owner = val.into();
        self
    }
    #[inline(always)]
    pub fn family_members(&self) -> &[String] {
        &self.family_members[..]
    }
    #[inline(always)]
    fn set_family_members<T: Into<String>>(
        &mut self,
        val: impl IntoIterator<Item = T>,
    ) -> &mut Self {
        self.family_members = val.into_iter().map(Into::into).collect();
        self
    }
    #[inline(always)]
    pub fn info(&self) -> &String {
        &self.info
    }
    #[inline(always)]
    fn set_info<T: Into<String>>(&mut self, val: T) -> &mut Self {
        self.info = val.into();
        self
    }
    #[inline(always)]
    pub(crate) fn mut_info(&mut self) -> &mut String {
        &mut self.info
    }
    #[inline(always)]
    pub fn note(&self) -> Option<&String> {
        self.note.as_ref()
    }
    #[inline(always)]
    fn set_note<T: Into<Option<String>>>(&mut self, val: T) -> &mut Self {
        self.note = val.into();
        self
    }
    #[inline(always)]
    pub fn note_mut(&mut self) -> &mut Option<String> {
        &mut self.note
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

## License

Licensed under either of [Apache License, Version 2.0] or [MIT License], at
your option.

[Apache License, Version 2.0]: LICENSE-APACHE
[MIT License]: LICENSE-MIT
