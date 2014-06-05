#![crate_type="lib"]
#![crate_id="burn#0.1"]
#![feature(macro_rules, struct_variant, globs)]
#![allow(unnecessary_parens)]

// this lint is a bit bugged
// https://github.com/mozilla/rust/pull/14413
#![allow(visible_private_types)]

extern crate core;
extern crate collections;
#[cfg(test)]
extern crate test;
extern crate debug;

pub use api::*;
mod api;

macro_rules! debug (
	( $b:stmt ) => { if unsafe { ::DEBUG } { $b } }
)

macro_rules! destructure_enum (
	( $e:expr to $p:pat => $b:block ) => {
		match $e {
			$p => $b,
			_ => { unreachable!(); }
		};
	}
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
	
	pub mod operations;
}

mod mem {
	pub mod raw;
	pub mod rc;
	pub mod gc;
}

mod vm {
	
	pub mod analysis {
		
		pub mod annotation;
		
		pub mod resolution;
		pub mod allocation;
	}
	
	pub mod bytecode {
		pub mod code;
		pub mod opcode;
		pub mod compiler;
	}
	
	pub mod run {
		pub mod fiber;
		pub mod flow;
		pub mod frame;
		pub mod rust;
		pub mod cpu;
	}
	
	pub mod virtual_machine;
	pub mod result;
	pub mod error;
	
	pub mod repl;
}

mod builtin {
	pub mod burn;
}

pub static mut DEBUG: bool = false;
