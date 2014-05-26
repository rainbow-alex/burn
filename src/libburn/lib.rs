#![crate_type="lib"]
#![crate_id="burn#0.1"]
#![feature(macro_rules, struct_variant, globs)]
#![allow(unnecessary_parens)]

// this lint is a but bugged
// https://github.com/mozilla/rust/pull/14413
#![allow(visible_private_types)]

extern crate core;
extern crate collections;
#[cfg(test)]
extern crate test;

pub use api::*;
mod api;

#[macro_export]
macro_rules! debug (
	( $b:stmt ) => { if unsafe { ::DEBUG } { $b } }
)

mod parse {
	
	pub mod token;
	pub mod node;
	
	pub mod lexer;
	pub mod parser;
	
	pub mod literal;
}

mod lang {
	
	pub mod identifier;
	
	pub mod module;
	pub mod function;
	pub mod type_;
	pub mod script;
	pub mod special;
	
	pub mod value;
}

mod mem {
	pub mod raw;
	pub mod rc;
	pub mod gc;
}

mod vm {
	
	pub mod analysis;
	
	pub mod bytecode {
		pub mod code;
		pub mod opcode;
		pub mod compiler;
	}
	
	pub mod virtual_machine;
	
	pub mod error;
	pub mod repl;
}

mod builtin {
	pub mod burn {
		pub mod implicit;
		pub mod operations;
		pub mod errors;
		pub mod types;
	}
}

pub static mut DEBUG: bool = false;
