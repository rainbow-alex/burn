use lang::script::Script;
use lang::function::Function;
use lang::value;
use mem::rc::Rc;
use mem::gc::Gc;
use vm::bytecode::code::Code;

pub struct Frame {
	pub type_: FrameType,
	pub local_variables: Vec<value::Value>,
	pub shared_local_variables: Vec<Rc<value::Value>>,
	pub instruction: uint,
}

pub enum FrameType {
	ScriptFrame( Box<Script> ),
	FunctionFrame( Gc<Function> ),
}

	impl Frame {
		
		pub fn new_script(
			script: Box<Script>,
			local_variables: Vec<value::Value>,
			shared_local_variables: Vec<Rc<value::Value>>
		) -> Frame {
			Frame {
				type_: ScriptFrame( script ),
				local_variables: local_variables,
				shared_local_variables: shared_local_variables,
				instruction: 0,
			}
		}
		
		pub fn new_function(
			function: Gc<Function>,
			local_variables: Vec<value::Value>,
			shared_local_variables: Vec<Rc<value::Value>>
		) -> Frame {
			Frame {
				type_: FunctionFrame( function ),
				local_variables: local_variables,
				shared_local_variables: shared_local_variables,
				instruction: 0,
			}
		}
		
		pub fn get_code<'l>( &'l self ) -> &'l Code {
			match self.type_ {
				ScriptFrame( ref script ) => &script.code,
				FunctionFrame( ref function ) => &function.get().definition.get().code,
			}
		}
		
		pub fn get_function( &self ) -> &mut Function {
			match self.type_ {
				ScriptFrame(..) => fail!(),
				FunctionFrame( ref function ) => function.get()
			}
		}
	}
