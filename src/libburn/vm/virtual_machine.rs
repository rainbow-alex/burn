use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use lang::function::Function;
use lang::module::Module;
use vm::bytecode::compiler;
use vm::run::fiber::Fiber;
use vm::run::cpu;
use vm::result;
use vm::repl;
use builtin::burn::{implicit, types, errors};

pub struct VirtualMachine {
	pub functions: GarbageCollectedManager<Function>,
	pub import_paths: Vec<Path>,
	pub module_root: Box<Module>,
	pub implicit: Raw<Module>,
}

	impl VirtualMachine {
		
		pub fn new() -> VirtualMachine {
			
			/*
			let mut root = box Module::new();
			let burn = box ::builtin::burn::create_module();
			root.add_module( "burn", burn );
			*/
			
			let mut root = box Module::new();
			let mut burn = box Module::new();
			
			let implicit = box implicit::create_module();
			let implicit_ref = Raw::new( implicit );
			burn.add_module( "implicit", implicit );
			
			burn.add_module( "errors", box errors::create_module() );
			burn.add_module( "types", box types::create_module() );
			
			burn.lock();
			root.add_module( "burn", burn );
			
			VirtualMachine {
				functions: GarbageCollectedManager::new(),
				import_paths: vec!( Path::new( "/home/alex/burn/modules/" ) ), // TODO
				module_root: root,
				implicit: implicit_ref,
				//implicit: Raw::new( root.get( "burn" ).get( "implicit" ) ),
			}
		}
		
		pub fn run_script( &mut self, source: &str ) -> result::Result {
			match compiler::compile_script( source ) {
				Err( errors ) => result::Fail( errors ),
				Ok( frame ) => {
					let fiber = box Fiber::new( frame );
					cpu::run( self, fiber )
				}
			}
		}
		
		pub fn run_repl( &mut self, repl_state: &mut repl::State, source: &str ) -> result::Result {
			match compiler::compile_repl( repl_state, source ) {
				Err( errors ) => result::Fail( errors ),
				Ok( frame ) => {
					let fiber = box Fiber::new( frame );
					cpu::run( self, fiber )
				}
			}
		}
	}
