// Copyright 2025 Pavel Roskin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Terminfo parsing library with simple API and minimal dependencies
//!
//! This crate provides facilities to
//!
//! * find the terminfo database for the given terminal
//! * parse the terminfo database and
//! * expand capabilities with parameters.
//!
//! Features:
//!
//! * full support for extended capabilities
//! * simple API
//! * extensive unit test coverage
//!
//! Why another terminfo library?
//!
//! * MIT + Apache 2.0 license (no obscenities or obscure licenses)
//! * minimal dependencies (`thiserror` only)
//! * truly lean - no termcap, no Windows console, no unrelated stuff
//! * UTF-8 is only used for capability names
//! * 8-bit clean - string capabilities are byte slices
//! * minimal memory allocations
//!
//! Credits
//!
//! The capability expansion code is based on the `term` crate with
//! significant changes.

pub mod expand;
pub mod locate;
pub mod parse;
