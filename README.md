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

  There are three kinds of configurable attributes: `get`, `set`, `mut`.

- Set container attributes can change the default settings for all fields.

- Change the settings of a single field via setting field attributes.

- The visibility of a method can be set via `#[property(get(visibility-type))]`

  There are four kinds of the visibility type: `disable`, `public`, `crate` (default for all methods), and `private`.

- The method name can be set in two ways:

  1. Assign a complete name via `#[property(get(name = "method-name"))]`.

  2. Set `prefix` and / or `suffix` via `#[property(set(prefix = "set_"), mut(suffix = "mut_"))]`.

  The default setting for all fields is: `#[property(get(prefix = "", suffix = ""), set(prefix = "set_"), mut(prefix = "mut_"))]`.

- The return type of `get` method can be set via `#[property(get(type = "return-type"))]`.

  There are three kinds of the return type: `ref` (default in most cases), `clone` and `copy`.

- The input type of `set` method can be set via `#[property(set(type = "input-type"))]`.

  There are three kinds of the input type: `ref` (default) and `own`.

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
#[property(get(public), set(private), mut(disable))]
pub struct Pet {
    #[property(get(name = "identification"), set(disable))]
    id: [u8; 32],
    name: String,
    #[property(set(crate, type = "own"))]
    age: u32,
    #[property(get(type = "copy"))]
    species: Species,
    #[property(get(prefix = "is_"))]
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
    fn set_name(&mut self, val: String) -> &mut Self {
        self.name = val;
        self
    }
    #[inline(always)]
    pub fn age(&self) -> u32 {
        self.age
    }
    #[inline(always)]
    pub(crate) fn set_age(mut self, val: u32) -> Self {
        self.age = val;
        self
    }
    #[inline(always)]
    pub fn species(&self) -> Species {
        self.species
    }
    #[inline(always)]
    fn set_species(&mut self, val: Species) -> &mut Self {
        self.species = val;
        self
    }
    #[inline(always)]
    pub fn is_died(&self) -> bool {
        self.died
    }
    #[inline(always)]
    fn set_died(&mut self, val: bool) -> &mut Self {
        self.died = val;
        self
    }
    #[inline(always)]
    pub fn owner(&self) -> String {
        self.owner.clone()
    }
    #[inline(always)]
    fn set_owner(&mut self, val: String) -> &mut Self {
        self.owner = val;
        self
    }
    #[inline(always)]
    pub fn family_members(&self) -> &[String] {
        &self.family_members[..]
    }
    #[inline(always)]
    fn set_family_members(&mut self, val: Vec<String>) -> &mut Self {
        self.family_members = val;
        self
    }
    #[inline(always)]
    pub fn info(&self) -> &String {
        &self.info
    }
    #[inline(always)]
    fn set_info(&mut self, val: String) -> &mut Self {
        self.info = val;
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
    fn set_note(&mut self, val: Option<String>) -> &mut Self {
        self.note = val;
        self
    }
    #[inline(always)]
    pub fn note_mut(&mut self) -> &mut Option<String> {
        &mut self.note
    }
}
```

Enjoy it!

## License

Licensed under either of [Apache License, Version 2.0] or [MIT License], at
your option.

[Apache License, Version 2.0]: LICENSE-APACHE
[MIT License]: LICENSE-MIT
