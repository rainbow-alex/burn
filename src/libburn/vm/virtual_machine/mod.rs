use collections::HashMap;
use mem::rc::Rc;
use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use lang::identifier::Identifier;
use lang::function::Function;
use lang::module::Module;
use lang::script::Script;
use lang::value;
use parse::parser;
use vm::bytecode::compiler;
use vm::error::Error;
use vm::virtual_machine::frame::Frame;
use vm::virtual_machine::fiber::Fiber;
use vm::virtual_machine::importing::ImportId;
use vm::repl;
use builtin::burn::{implicit, types, errors};

pub mod flow;
pub mod frame;
pub mod fiber;

pub mod result {
	use lang::value;
	use vm::error::Error;
	
	pub enum Result {
		End,
		Errors( Vec<Box<Error>> ),
		Value( value::Value ),
		UncaughtThrowable( value::Value ),
	}
}

pub struct VirtualMachine {
	functions: GarbageCollectedManager<Function>,
	module_root: Box<Module>,
	implicit: Raw<Module>,
	import_paths: Vec<Path>,
	import_ids_by_fqn: HashMap<Vec<Identifier>,ImportId>,
	import_cache: Vec<importing::ImportCacheEntry>,
}

	impl VirtualMachine {
		
		pub fn new() -> VirtualMachine {
			
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
				module_root: root,
				implicit: implicit_ref,
				import_paths: vec!( Path::new( "/home/alex/burn/src/modules/" ) ),
				import_ids_by_fqn: HashMap::new(),
				import_cache: Vec::new(),
			}
		}
		
		pub fn run_script( &mut self, code: &str ) -> result::Result {
			
			let mut ast = match parser::parse_script( code ) {
				Ok( ast ) => ast,
				Err( error ) => {
					return result::Errors( vec!( box error as Box<Error> ) );
				}
			};
			
			let code = match compiler::compile_script( ast ) {
				Ok( code ) => code,
				Err( errors ) => {
					return result::Errors( errors.move_iter().map( |e| { box e as Box<Error> } ).collect() );
				}
			};
			
			let fiber = {
				let locals = Vec::from_elem( code.n_local_variables, value::Nothing );
				let shared = Vec::from_fn( code.n_shared_local_variables, |_| { Rc::new( value::Nothing ) } );
				let script = box Script { code: code };
				let frame = Frame::new_script( script, locals, shared );
				
				Fiber::new( frame )
			};
			
			self.run_fiber( fiber )
		}
		
		pub fn run_repl( &mut self, repl_state: &mut repl::State, source: &str ) -> result::Result {
			
			let mut ast = match parser::parse_repl( source ) {
				Ok( ast ) => ast,
				Err( error ) => {
					return result::Errors( vec!( box error as Box<Error> ) );
				}
			};
			
			let code = match compiler::compile_repl( ast, repl_state ) {
				Ok( code ) => code,
				Err( errors ) => {
					return result::Errors( errors.move_iter().map( |e| { box e as Box<Error> } ).collect() );
				}
			};
			
			let locals = Vec::from_elem( code.n_local_variables, value::Nothing );
			let mut shared = Vec::from_fn( code.n_shared_local_variables, |_| { Rc::new( value::Nothing ) } );
			
			for variable in ast.root.frame.declared.iter().take( repl_state.variables.len() ) {
				*shared.get_mut( variable.local_storage_index ) = repl_state.variables.find( &variable.name ).unwrap().clone();
			}
			
			let script = box Script { code: code };
			let frame = Frame::new_script( script, locals, shared );
			
			let fiber = Fiber::new( frame );
			
			self.run_fiber( fiber )
		}
	}

#[doc(hidden)]
trait VirtualMachineRunFiber {
	fn run_fiber( &mut self, fiber: Fiber ) -> result::Result;
}

#[doc(hidden)]
trait VirtualMachineImporting {
	fn find_import( &mut self, fqn: Vec<Identifier> ) -> (ImportId, bool);
	fn import_or_get_cached( &mut self, id: ImportId ) -> Result<value::Value,value::Value>;
}

mod run_fiber;
mod importing;
