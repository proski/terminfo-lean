// Copyright 2025 Pavel Roskin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Terminfo parsing library with simple API and minimal dependencies
//!
//! ## Provided Functionality
//!
//! * Find the terminfo database for the given terminal.
//! * Parse the terminfo database.
//! * Expand capabilities with parameters.
//!
//! ## Why another terminfo library?
//!
//! * Full support for extended capabilities
//! * MIT + Apache 2.0 license (no obscene or obscure licenses)
//! * Extensive unit test coverage
//! * Minimal dependencies (`thiserror` only)
//! * Lean code - no termcap, no Windows console, no unrelated stuff
//! * UTF-8 is only used for capability names
//! * 8-bit clean - string capabilities are byte slices
//! * Minimal memory allocations
//!
//! ## Credits
//!
//! The capability expansion code is based on the `term` crate with
//! significant changes.

pub mod expand;
pub mod locate;
pub mod parse;
