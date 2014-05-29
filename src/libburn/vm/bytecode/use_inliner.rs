use mem::raw::Raw;
use lang::identifier::Identifier;
use vm::bytecode::code::Code;
use vm::bytecode::opcode;
use vm::run::rust;

pub struct UseInliner {
	path: Vec<Identifier>,
	usages: Vec<(Raw<Code>, uint)>,
}

	impl UseInliner {
		
		pub fn new( path: Vec<Identifier> ) -> UseLoader {
			UseLoader {
				path: path,
				inlines: Vec::new(),
			}
		}
		
		pub fn add_name_opcode( &mut self, code: Raw<Code>, offset: uint ) {
			self.usages.push( (code, offset) );
		}
	}
	
	impl rust::Operation for UseInliner {
		pub fn run( &mut self, value: value::Value ) -> rust::Result {
			
			let opcode = opcode::PushNothing; // TODO
			
			for (code, offset) in self.usages.iter() {
				*code.get().opcodes.get_mut( offset ) = opcode;
			}
		}
	}
