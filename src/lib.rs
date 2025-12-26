// Copyright 2025 Pavel Roskin
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Terminfo parsing library with simple API and minimal dependencies

pub mod expand;
pub mod locate;
pub mod parse;

pub use expand::{ExpandContext, Parameter};
pub use locate::{locate, search_directories};
pub use parse::Terminfo;
