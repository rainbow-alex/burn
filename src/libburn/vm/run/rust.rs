use lang::value::Value;
use vm::run::frame::Frame;
use vm::virtual_machine::VirtualMachine;

pub trait Operation {
	fn run( &mut self, &mut VirtualMachine, ::std::result::Result<Value,Value> ) -> Result;
}

pub enum Result {
	Ok( Value ),
	Throw( Value ),
	TailBurn( Frame ),
	TailRust( Box<Operation> ),
	TailYield,
	Burn( Frame ),
	Rust( Box<Operation> ),
	Yield,
}
