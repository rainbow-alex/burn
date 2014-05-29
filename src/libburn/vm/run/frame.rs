use mem::rc::Rc;
use mem::gc::Gc;
use lang::value;
use lang::script::Script;
use lang::function::Function;
use vm::bytecode::code::Code;
use vm::run::rust;

pub struct Frame {
	pub type_: FrameType,
	pub local_variables: Vec<value::Value>,
	pub shared_local_variables: Vec<Rc<value::Value>>,
	pub instruction: uint,
}

pub enum FrameType {
	Main( Script ),
	Function( Gc<Function> ),
	Rust( Box<rust::Operation> ),
}

	impl Frame {
		
		pub fn new_builtin( operation: Box<rust::Operation> ) -> Frame {
			Frame {
				type_: Rust( operation ),
				local_variables: Vec::new(),
				shared_local_variables: Vec::new(),
				instruction: 0,
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
	}
