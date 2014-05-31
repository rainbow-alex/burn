use mem::rc::Rc;
use mem::gc::Gc;
use lang::value;
use lang::script::Script;
use lang::function::Function;
use vm::bytecode::code::Code;
use vm::run::rust;

pub struct Frame {
	type_: FrameType,
	local_variables: Vec<value::Value>,
	shared_local_variables: Vec<Rc<value::Value>>,
	pub instruction: uint,
}

pub enum FrameType {
	Main( Script ),
	Function( Gc<Function> ),
	Rust( Box<rust::Operation> ),
}

	impl Frame {
		
		pub fn new_main( script: Script, locals: Vec<value::Value>, shared: Vec<Rc<value::Value>> ) -> Frame {
			Frame {
				type_: Main( script ),
				local_variables: locals,
				shared_local_variables: shared,
				instruction: 0,
			}
		}
		
		pub fn new_function( function: Gc<Function>, locals: Vec<value::Value>, shared: Vec<Rc<value::Value>> ) -> Frame {
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
				Function( ref mut function ) => &mut *function.get().definition.get().code,
				Rust(..) => fail!(),
			}
		}
		
		pub fn get_closure( &self ) -> &mut Function {
			match self.type_ {
				Main(..) => fail!(),
				Function( ref function ) => function.get(),
				Rust(..) => fail!(),
			}
		}
		
		pub fn get_rust_operation<'l>( &'l mut self ) -> &'l mut Box<rust::Operation> {
			match self.type_ {
				Rust( ref mut operation ) => operation,
				_ => { fail!(); }
			}
		}
		
		pub fn get_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut value::Value {
			self.local_variables.get_mut( index )
		}
		
		pub fn get_shared_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut value::Value {
			self.shared_local_variables.get_mut( index ).get()
		}
	}
