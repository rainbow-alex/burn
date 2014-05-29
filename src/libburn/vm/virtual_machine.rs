use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use lang::function::Function;
use lang::module::Module;
use vm::bytecode::compiler;
use vm::run::fiber::Fiber;
use vm::run::cpu;
use vm::result;
use vm::repl;

pub struct VirtualMachine {
	pub functions: GarbageCollectedManager<Function>,
	pub import_paths: Vec<Path>,
	pub module_root: Box<Module>,
	pub implicit: Raw<Module>,
}

	impl VirtualMachine {
		
		pub fn new() -> VirtualMachine {
			
			let mut root = box Module::new();
			let burn = box ::builtin::burn::create_module();
			root.add_module( "burn", burn );
			
			VirtualMachine {
				functions: GarbageCollectedManager::new(),
				import_paths: vec!( Path::new( "modules/" ) ), // TODO
				implicit: Raw::new( root.get_module( "burn" ).get_module( "implicit" ) ),
				module_root: root,
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
