#![crate_name = "jade"]
#![feature(collections)]
#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(unused_variables)]

extern crate regex;

// This rewrites the regex! macro while compiler
// extensions are not in stable
macro_rules! regex(
    ($s:expr) => (regex::Regex::new($s).unwrap());
);

pub mod lexer;
pub mod brackets;

pub fn parse(tpl: String) {
    
}

#[test]
#[ignore]
fn parse_simple() {
}
