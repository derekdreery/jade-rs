#![crate_name = "jade"]
#![comment = "Jade html templating from NodeJs"]
#![license = "MIT"]
#![feature(slicing_syntax)]
#![feature(phase)]
#![allow(dead_code)]
#![allow(unused_attributes)]
#![allow(unused_variables)]

extern crate regex;
#[phase(plugin)] extern crate regex_macros;

mod tokens;
mod lexer;

pub fn parse(tpl: String) {
    
}

#[test]
fn it_works() {
}
