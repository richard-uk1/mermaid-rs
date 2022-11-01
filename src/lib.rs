//! A port of the [`mermaid.js`](https://mermaid-js.github.io/mermaid/#/) chart drawing library to
//! Rust.
//!
//! # Goals
//!
//! - User friendly
//!   - Very forgiving grammars that uses backtracking where necessary to accept as many different
//!   valid inputs as possible.
//!   - Error messages that explain the problem and locate it in the input.
//! - Optional rendering of charts using [`piet`](https://crates.io/crates/piet).
//!
//! # Non-goals
//!
//!  - Exact 1-1 correspondence between accepted grammars of `mermaid.js` and this library.
//!  - Exact 1-1 look of rendered charts between `mermaid.js` and this library.

mod diagrams;
pub mod style;

pub use diagrams::*;
