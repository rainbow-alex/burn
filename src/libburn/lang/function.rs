use std::vec::Vec;
use mem::rc::{Rc, RefCounted};
use mem::gc::GarbageCollected;
use lang::value;
use lang::identifier::Identifier;
use vm::bytecode::code::Code;

pub struct Function {
	#[doc(hidden)]
	pub definition: Rc<FunctionDefinition>,
	#[doc(hidden)]
	pub static_bound_variables: Vec<value::Value>,
	#[doc(hidden)]
	pub shared_bound_variables: Vec<Rc<value::Value>>,
}

	impl Function {
		
		pub fn new(
			definition: Rc<FunctionDefinition>
		) -> Function {
			
			let n_bindings = definition.get().bindings.len();
			let n_static = definition.get().n_static_bound_variables;
			let n_shared = n_bindings - n_static;
			
			Function {
				definition: definition,
				static_bound_variables: Vec::from_elem( n_static, value::Nothing ),
				shared_bound_variables: Vec::from_fn( n_shared, |_| { Rc::new( value::Nothing ) } ),
			}
		}
	}
	
	impl GarbageCollected for Function {
		
		fn mark( &mut self ) {
			not_implemented!();
		}
	}

pub struct FunctionDefinition {
	pub parameters: Vec<FunctionParameterDefinition>,
	pub bindings: Vec<FunctionBindingDefinition>,
	pub code: Box<Code>,
	pub n_static_bound_variables: uint,
}

	impl FunctionDefinition {
		
		pub fn new(
			code: Box<Code>,
			parameters: Vec<FunctionParameterDefinition>,
			bindings: Vec<FunctionBindingDefinition>
		) -> FunctionDefinition {
			
			let n_static_bound_variables = bindings.iter().filter( |b| {
				match **b {
					LocalToStaticBoundBinding(..) | StaticBoundToStaticBoundBinding(..) => true,
					_ => false,
				}
			} ).len();
			
			FunctionDefinition {
				parameters: parameters,
				bindings: bindings,
				code: code,
				n_static_bound_variables: n_static_bound_variables,
			}
		}
	}
	
	impl RefCounted for FunctionDefinition {}

pub struct FunctionParameterDefinition {
	pub name: Identifier,
	pub storage: FunctionParameterStorage,
}

pub enum FunctionParameterStorage {
	LocalFunctionParameterStorage( uint ),
	SharedLocalFunctionParameterStorage( uint ),
}

pub enum FunctionBindingDefinition {
	LocalToStaticBoundBinding( uint, uint ),
	SharedLocalToSharedBoundBinding( uint, uint ),
	StaticBoundToStaticBoundBinding( uint, uint ),
	SharedBoundToSharedBoundBinding( uint, uint ),
}
