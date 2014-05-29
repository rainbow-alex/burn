use lang::value;
use vm::error::Error;
use vm::run::frame::Frame;
use vm::virtual_machine::VirtualMachine;

pub trait Operation {
	fn run( &mut self, &mut VirtualMachine, ::std::result::Result<value::Value,value::Value> ) -> Result;
}

pub enum Result {
	Ok( value::Value ),
	Throw( value::Value ),
	Burn( Frame ),
	Rust( Box<Operation> ),
	Yield,
	Fail( Vec<Box<Error>> ),
}
