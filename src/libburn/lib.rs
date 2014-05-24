#![crate_type="lib"]
#![crate_id="burn#0.1"]
#![feature(macro_rules, struct_variant)]
#![allow(unnecessary_parens)]

extern crate core;
extern crate collections;
#[cfg(test)]
extern crate test;

pub mod error;

mod parse {
	pub mod token;
	pub mod node;
	
	pub mod lexer;
	pub mod parser;
	
	pub mod literal;
}

mod compile {
	pub mod analysis;
	pub mod compiler;
}

pub mod lang {
	
	pub mod string;
	pub mod identifier;
	
	pub mod module;
	pub mod function;
	pub mod type_;
	pub mod script;
	pub mod special;
	
	pub mod value;
}

pub mod mem {
	pub mod raw;
	pub mod rc;
	pub mod gc;
}

pub mod vm {
	pub mod code;
	pub mod opcode;
	pub mod fiber;
	pub mod frame;
	pub mod flow;
	
	pub mod result;
	pub mod virtual_machine;
	
	pub mod repl;
}

pub mod builtin {
	pub mod intrinsic;
}

pub static mut DEBUG_BYTECODE: bool = false;
