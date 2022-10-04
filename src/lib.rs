//! Rust enums are coproducts but the datastructure provided in this library
//! allows writing functions that operate on generic coproducts.
//!
//! For instance, the below function takes any coproduct that may contain a cat.
//! ```
//! # use coproduct::{Coproduct, Count, IndexedDrop};
//! # struct Cat;
//! fn is_cat<C, I>(maybe_cat: C) -> bool
//! where
//!     C: coproduct::At<I, Cat>,
//! {
//!     maybe_cat.uninject().is_ok()
//! }
//! ```
//!
//! The coproducts take as much memory as the largest variant and 32 bits
//! for the tag, which is pretty close to optimal. They do not benefit from
//! Rust's enum layout optimizations, but the whole reason for this crate is
//! that those optimizations aren't perfect. Implementing a coproduct as nested
//! enums akin to a purely functional list results in extremely high memory use.
//! (Tested in Rust 1.66)
//!
//! Another benefit is that the implementation of some functions is a lot simpler
//! when there is no need to pretend that a nested structure is traversed. The
//! downside is that unlike the coproduct provided by frunk, this library uses
//! unsafe.

mod coproduct;
mod count;
pub mod merge;
mod public_traits;
pub mod type_inequality;
mod union;

pub use crate::coproduct::*;
pub use count::*;
pub use merge::Merge;
pub use public_traits::*;
pub use union::{EmptyUnion, Union};
