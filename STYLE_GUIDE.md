# Order Within a File

```rust
//! Module documentation

// Extern crates (rare in modern Rust)
extern crate some_extern_crate;

// Re-exports and imports.
// Grouped by visibility and crate.
// Sorted alphabetically.
pub use crate::somecrate::*;
pub(crate) use crate::foo::bar;

use std::fmt;

use third_party::crate;

use crate::foo::bar;

use super::baz;

// Module declarations
mod internal;
pub mod public_api;

// Constants / statics.
pub const MAX: usize = 100;
pub(crate) const INTERNAL_MAX: usize = 50;

// Type aliases
type MyType = u32;

// Structs / Enums

pub struct PublicStruct {
    pub field: u32,
    pub(crate) crate_field: u32,
    pub(super) super_field: u32,
    private_field: u32,
}

pub(crate) struct CrateStruct {
    pub field: u32,
    pub(crate) crate_field: u32,
    pub(super) super_field: u32,
    private_field: u32,
}

struct PrivateStruct {
    pub field: u32,
    pub(crate) crate_field: u32,
    pub(super) super_field: u32,
    private_field: u32,
}

// Traits

pub trait MyTrait {
    fn do_something(&self);
}

// Impls

impl MyTrait for PublicStruct {
    fn do_something(&self) {
        // ...
    }
}

impl PublicStruct {

    pub fn new() -> Self {
        Self {
            field: 0,
            crate_field: 0,
            super_field: 0,
            private_field: 0,
        }
    }

    pub(crate) fn crate_only(&self) {}

    pub(super) fn super_only(&self) {}

    fn private_helper(&self) {}
}

/// Functions

pub fn public_function() {}

pub(crate) fn crate_function() {}

pub(super) fn super_function() {}

fn private_function() {}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```