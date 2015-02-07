#![crate_name = "jade"]
#![comment = "Jade html templating from NodeJs"]
#![license = "MIT"]
#![feature(slicing_syntax)]
#![feature(plugin)]
#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(unused_variables)]
#![allow(unstable)]

extern crate core;
extern crate regex;
#[plugin] extern crate regex_macros;

pub mod lexer;
pub mod brackets;

pub fn parse(tpl: String) {
    
}

#[test]
#[ignore]
fn parse_simple() {
}
