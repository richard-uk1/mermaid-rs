// This is a template for using lalrpop with logos.
//
// You need to know how they both work to edit this template, but having it here can save some
// boilerplate work.
#[macro_use]
extern crate lalrpop_util;

mod diagrams;
mod parser_utils;

pub use diagrams::*;
