use std::vec::Vec;
use lang::value;
use lang::identifier::Identifier;
use mem::rc::{Rc, RcHeader, RefCounted};
use mem::gc::{GcHeader, GarbageCollected};
use vm::code::Code;

pub struct Function {
	gc: GcHeader,
	pub definition: Rc<FunctionDefinition>,
	pub static_bound_variables: Vec<value::Value>,
	pub shared_bound_variables: Vec<value::SharedValue>,
}

	impl Function {
		
		pub fn new(
			definition: Rc<FunctionDefinition>
		) -> Function {
			Function {
				gc: GcHeader::new(),
				definition: definition,
				static_bound_variables: Vec::new(), // TODO
				shared_bound_variables: Vec::new(), // TODO
			}
		}
	}
	
	impl GarbageCollected for Function {
		
		fn get_gc_header<'l>( &'l mut self ) -> &'l mut GcHeader {
			&'l mut self.gc
		}
	}

pub struct FunctionDefinition {
	rc: RcHeader,
	pub parameters: Vec<FunctionParameterDefinition>,
	pub code: Code,
}

	impl FunctionDefinition {
		
		pub fn new( parameters: Vec<FunctionParameterDefinition>, code: Code ) -> FunctionDefinition {
			FunctionDefinition {
				rc: RcHeader::new(),
				parameters: parameters,
				code: code,
			}
		}
	}
	
	impl RefCounted for FunctionDefinition {
		
		fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
			&mut self.rc
		}
	}

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
