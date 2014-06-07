use rustuv::uvll;
use libc::c_void;
use std::mem;

use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use lang::function::Function;
use lang::module::Module;
use lang::value::Value;
use vm::bytecode::compiler;
use vm::run::fiber::Fiber;
use vm::run::cpu;
use vm::repl;

pub trait UncaughtThrowableHandler {
	fn handle_uncaught_throwable( &mut self, &mut VirtualMachine, Value );
}

pub struct VirtualMachine {
	pub uv_loop: *c_void,
	pub functions: GarbageCollectedManager<Function>,
	pub import_paths: Vec<Path>,
	pub module_root: Box<Module>,
	pub implicit: Raw<Module>,
	pub uncaught_throwable_handlers: Vec<Box<UncaughtThrowableHandler>>,
}

	impl VirtualMachine {
		
		pub fn new() -> VirtualMachine {
			
			let mut root = box Module::new();
			let burn = box ::builtin::burn::create_module();
			root.add_module( "burn", burn );
			
			VirtualMachine {
				uv_loop: unsafe { uvll::loop_new() },
				functions: GarbageCollectedManager::new(),
				import_paths: vec!( Path::new( "modules/" ) ), // todo!
				implicit: Raw::new( root.get_module( "burn" ).get_module( "implicit" ) ),
				module_root: root,
				uncaught_throwable_handlers: Vec::new(),
			}
		}
		
		pub fn schedule( &mut self, f: proc( &mut VirtualMachine ) ) {
			
			let vm_ptr: *() = unsafe { mem::transmute( &*self ) };
			let f_ptr: *() = unsafe { mem::transmute( box f ) };
			
			// optimize! try using a boxed tuple instead of binding into a new proc?
			let f = box proc() {
				let vm: &mut VirtualMachine = unsafe { mem::transmute( vm_ptr ) };
				let f: Box<proc( &mut VirtualMachine )> = unsafe { mem::transmute( f_ptr ) };
				(*f)( vm );
			};
			
			let handle = ::rustuv::UvHandle::alloc( None::<::rustuv::AsyncWatcher>, uvll::UV_ASYNC );
			unsafe {
				uvll::set_data_for_uv_handle( handle, &*f );
				mem::forget( f );
				uvll::uv_async_init( self.uv_loop, handle, callback );
				uvll::uv_async_send( handle );
			}
			
			extern "C" fn callback( handle: *uvll::uv_async_t ) {
				let f: Box<proc()> = unsafe { mem::transmute( uvll::get_data_for_uv_handle( handle ) ) };
				(*f)();
			}
		}
		
		pub fn schedule_fiber( &mut self, fiber: Box<Fiber> ) {
			self.schedule( proc( vm ) {
				cpu::run( vm, fiber );
			} );
		}
		
		pub fn run( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_ONCE ); }
			// todo! uv_loop_delete
		}
		
		pub fn run_loop( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_DEFAULT ); }
			// todo! uv_loop_delete
		}
		
		pub fn run_script( &mut self, source: &str ) {
			match compiler::compile_script( source ) {
				Ok( frame ) => {
					let fiber = box Fiber::new( frame );
					cpu::run( self, fiber );
				}
				Err( errors ) => {
					(errors);
					unimplemented!();
				}
			}
		}
		
		pub fn run_repl( &mut self, repl_state: &mut repl::State, source: &str ) {
			match compiler::compile_repl( repl_state, source ) {
				Ok( frame ) => {
					let fiber = box Fiber::new( frame );
					cpu::run( self, fiber );
				}
				Err( errors ) => {
					(errors);
					unimplemented!();
				}
			}
		}
		
		pub fn to_string( &mut self, value: Value ) -> Result<String,()> {
			Ok( "todo".to_string() )
			
			/* todo!
			let mut result = box String::new();
			let result_ptr = &mut *result as *mut String;
			
			let source = "";
			let frame = compiler::compile_script( source ).ok().unwrap();
			let mut fiber = box Fiber::new( frame );
			fiber.on_return = Some( proc( to_string_value: Value ) {
				match to_string_value {
					::lang::value::String( s ) => {
						
						unsafe { *result_ptr = s.borrow().to_string(); }
					}
					_ => { unreachable!(); }
				}
			} );
			cpu::run( self, fiber );
			self.run();
			
			Ok( *result )
			*/
		}
		
		pub fn on_uncaught_throwable( &mut self, handler: Box<UncaughtThrowableHandler> ) {
			self.uncaught_throwable_handlers.push( handler );
		}
	}
