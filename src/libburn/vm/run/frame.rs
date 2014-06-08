use mem::rc::Rc;
use mem::gc::Gc;
use lang::value::Value;
use lang::origin::Origin;
use lang::function::Function;
use vm::bytecode::code::Code;
use vm::run::rust;

pub enum Frame {
	
	BurnRootFrame {
		pub origin: Rc<Box<Origin>>,
		pub code: Box<Code>,
		pub context: BurnContext,
	},
	
	BurnFunctionFrame {
		pub function: Gc<Function>,
		pub context: BurnContext,
	},
	
	RustOperationFrame( Box<rust::Operation> ),
}

	impl Frame {
		
		pub fn is_rust_operation( &self ) -> bool {
			match *self {
				RustOperationFrame(..) => true,
				_ => false,
			}
		}
		
		pub fn get_code<'l>( &'l mut self ) -> &'l mut Code {
			match *self {
				BurnRootFrame { code: ref mut code, .. } => &mut **code,
				BurnFunctionFrame { function: ref mut function, .. } => &mut *function.definition.code,
				
				RustOperationFrame(..) => { unreachable!(); }
			}
		}
		
		// optimize! unsafe get for performance?
		pub fn get_context<'l>( &'l mut self ) -> &'l mut BurnContext {
			match *self {
				BurnRootFrame { context: ref mut context,.. }
				| BurnFunctionFrame { context: ref mut context, .. }
				=> context,
				
				RustOperationFrame(..) => { unreachable!(); }
			}
		}
		
		pub fn get_rust_operation<'l>( &'l mut self ) -> &'l mut Box<rust::Operation> {
			match *self {
				RustOperationFrame( ref mut operation ) => operation,
				_ => { unreachable!(); }
			}
		}
		
		pub fn get_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut Value {
			self.get_context().local_variables.get_mut( index )
		}
		
		pub fn get_shared_local_variable<'l>( &'l mut self, index: uint ) -> &'l mut Option<Rc<Value>> {
			self.get_context().shared_local_variables.get_mut( index )
		}
		
		fn get_closure<'l>( &'l mut self ) -> &'l mut Function {
			match_enum!( *self to BurnFunctionFrame { function: ref mut function, .. } => { &mut **function } )
		}
		
		pub fn get_static_bound_variable<'l>( &'l mut self, index: uint ) -> &'l mut Value {
			self.get_closure().static_bound_variables.get_mut( index )
		}
		
		pub fn get_shared_bound_variable<'l>( &'l mut self, index: uint ) -> &'l mut Rc<Value> {
			self.get_closure().shared_bound_variables.get_mut( index )
		}
	}

type Locals = Vec<Value>;
type SharedLocals = Vec<Option<Rc<Value>>>;

pub struct BurnContext {
	// optimize! the length of *_variables is known via the type
	// so instead of a vec, this could be a pointer to a fixed-size buffer
	pub local_variables: Locals,
	// optimize! someday, rust should be able to store Option<Rc<...>> in one word
	pub shared_local_variables: SharedLocals,
	pub instruction: uint,
}

	impl BurnContext {
		
		pub fn new( locals: Locals, shared: SharedLocals ) -> BurnContext {
			BurnContext {
				local_variables: locals,
				shared_local_variables: shared,
				instruction: 0,
			}
		}
	}
