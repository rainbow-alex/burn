use mem::rc::Rc;
use mem::gc::Gc;
use lang::value::Value;
use lang::script::Script;
use lang::function::Function;
use vm::bytecode::code::Code;
use vm::run::rust;

pub struct Frame {
	type_: FrameType,
	local_variables: Vec<Value>,
	// optimize! someday, rust should be able to store this in one word
	shared_local_variables: Vec<Option<Rc<Value>>>,
	pub instruction: uint,
}

enum FrameType {
	Main( Script ),
	Function( Gc<Function> ),
	Rust( Box<rust::Operation> ),
}

	impl Frame {
		
		pub fn new_main( script: Script, locals: Vec<Value>, shared: Vec<Option<Rc<Value>>> ) -> Frame {
			Frame {
				type_: Main( script ),
				local_variables: locals,
				shared_local_variables: shared,
				instruction: 0,
			}
		}
		
		pub fn new_function( function: Gc<Function>, locals: Vec<Value>, shared: Vec<Option<Rc<Value>>> ) -> Frame {
			Frame {
				type_: Function( function ),
				local_variables: locals,
				shared_local_variables: shared,
				instruction: 0,
			}
		}
		
		pub fn new_rust( operation: Box<rust::Operation> ) -> Frame {
			Frame {
				type_: Rust( operation ),
				local_variables: Vec::new(),
				shared_local_variables: Vec::new(),
				instruction: 0,
			}
		}
		
		pub fn is_rust( &self ) -> bool {
			match self.type_ {
				Rust(..) => true,
				_ => false,
			}
		}
		
		pub fn get_code<'l>( &'l mut self ) -> &'l mut Code {
			match self.type_ {
				Main( ref mut script ) => &mut *script.code,
				Function( ref mut function ) => &mut *function.borrow().definition.borrow().code,
				Rust(..) => unreachable!(),
			}
		}
		
		pub fn get_rust_operation<'l>( &'l mut self ) -> &'l mut Box<rust::Operation> {
			match self.type_ {
				Rust( ref mut operation ) => operation,
				_ => { unreachable!(); }
			}
		}
		
		pub fn get_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut Value {
			self.local_variables.get_mut( index )
		}
		
		pub fn get_shared_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut Option<Rc<Value>> {
			self.shared_local_variables.get_mut( index )
		}
		
		fn get_closure<'l>( &'l self ) -> &'l mut Function {
			match_enum!( self.type_ to Function( ref function ) => { function.borrow() } )
		}
		
		pub fn get_static_bound_variable<'l>( &'l mut self, index: uint ) -> &'l mut Value {
			self.get_closure().static_bound_variables.get_mut( index )
		}
		
		pub fn get_shared_bound_variable<'l>( &'l mut self, index: uint ) -> &'l mut Rc<Value> {
			self.get_closure().shared_bound_variables.get_mut( index )
		}
	}
