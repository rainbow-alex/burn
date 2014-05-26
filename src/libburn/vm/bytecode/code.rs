use mem::rc::Rc;
use lang::function::FunctionDefinition;
use vm::bytecode::opcode;

pub struct Code {
	pub n_local_variables: uint,
	pub n_shared_local_variables: uint,
	pub opcodes: Vec<opcode::OpCode>,
	pub strings: Vec<Rc<String>>,
	pub functions: Vec<Rc<FunctionDefinition>>,
}

	impl Code {
		
		pub fn new() -> Code {
			Code {
				n_local_variables: 0,
				n_shared_local_variables: 0,
				opcodes: Vec::new(),
				strings: Vec::new(),
				functions: Vec::new(),
			}
		}
		
		pub fn dump( &self ) {
			println!( "\\{" );
			self.dump_indented( "" );
		}
		
		fn dump_indented( &self, indent: &str ) {
			println!( "{}  n_local_variables: {}", indent, self.n_local_variables );
			println!( "{}  n_shared_local_variables: {}", indent, self.n_shared_local_variables );
			println!( "{}  opcodes: {}", indent, self.opcodes.len() );
			for (i, c) in self.opcodes.iter().enumerate() {
				println!( "{}    {}: {}", indent, i, c );
			}
			println!( "{}  strings: {}", indent, self.strings.len() );
			println!( "{}  functions: {}", indent, self.functions.len() );
			for (i, f) in self.functions.iter().enumerate() {
				println!( "{}    {}: \\{", indent, i );
				f.get().code.dump_indented( indent.to_owned().append( "    " ).as_slice() );
			}
			println!( "{}\\}", indent );
		}
	}
