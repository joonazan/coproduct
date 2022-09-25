//! Rust enums are coproducts but the datastructure provided in this library
//! allows writing functions that operate on generic coproducts.
//!
//! For instance, the below function takes any coproduct containing a cat.
//! ```
//! // TODO
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
mod counter;
mod union;
