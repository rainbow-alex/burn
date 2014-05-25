use std::vec::Vec;
use lang::value;
use lang::identifier::Identifier;
use mem::rc::{Rc, RefCounted};
use mem::gc::GarbageCollected;
use vm::code::Code;

pub struct Function {
	pub definition: Rc<FunctionDefinition>,
	pub static_bound_variables: Vec<value::Value>,
	pub shared_bound_variables: Vec<value::SharedValue>,
}

	impl Function {
		
		pub fn new(
			definition: Rc<FunctionDefinition>
		) -> Function {
			Function {
				definition: definition,
				static_bound_variables: Vec::new(), // TODO
				shared_bound_variables: Vec::new(), // TODO
			}
		}
	}
	
	impl GarbageCollected for Function {
		
		fn mark( &mut self ) {
			// TODO
		}
	}

pub struct FunctionDefinition {
	pub parameters: Vec<FunctionParameterDefinition>,
	pub code: Code,
	pub n_static_bound_variables: uint,
	pub n_shared_bound_variables: uint,
}

	impl FunctionDefinition {
		
		pub fn new( parameters: Vec<FunctionParameterDefinition>, code: Code ) -> FunctionDefinition {
			FunctionDefinition {
				parameters: parameters,
				code: code,
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			}
		}
	}
	
	impl RefCounted for FunctionDefinition {}

pub struct FunctionParameterDefinition {
	type_: Option<Code>,
	name: Identifier,
	default: Option<Code>,
	storage: FunctionParameterStorage,
}

pub enum FunctionParameterStorage {
	FreeFunctionParameterStorage { index: uint },
	SharedFunctionParameterStorage { index: uint },
}
