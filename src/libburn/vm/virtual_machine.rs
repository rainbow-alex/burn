use rustuv::uvll;
use libc::c_void;
use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use lang::function::Function;
use lang::module::Module;
use lang::value::Value;
use vm::run::fiber::Fiber;
use vm::error::{Error, UncaughtThrowableHandler};
use vm::repl;

pub struct VirtualMachine {
	pub functions: GarbageCollectedManager<Function>,
	pub import_paths: Vec<Path>,
	pub module_root: Box<Module>,
	pub implicit: Raw<Module>,
	pub uv_loop: *c_void,
	pub uncaught_throwable_handlers: Vec<Box<UncaughtThrowableHandler>>,
}

	impl VirtualMachine {
		
		pub fn new() -> VirtualMachine {
			
			let mut root = box Module::new();
			let burn = box ::builtin::burn::create_module();
			root.add_module( "burn", burn );
			
			VirtualMachine {
				functions: GarbageCollectedManager::new(),
				import_paths: vec!( Path::new( "modules/" ) ), // todo!
				implicit: Raw::new( root.get_module( "burn" ).get_module( "implicit" ) ),
				module_root: root,
				uv_loop: unsafe { uvll::loop_new() },
				uncaught_throwable_handlers: Vec::new(),
			}
		}
		
		pub fn on_uncaught_throwable( &mut self, handler: Box<UncaughtThrowableHandler> ) {
			self.uncaught_throwable_handlers.push( handler );
		}
		
		pub fn run( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_ONCE ); }
		}
		
		pub fn run_loop( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_DEFAULT ); }
		}
		
		pub fn schedule( &mut self, f: proc( &mut VirtualMachine ) ) {
			
			use std::mem;
			
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
				use vm::run::cpu;
				cpu::run( vm, fiber );
			} );
		}
		
		pub fn schedule_script( &mut self, source: &str ) -> Result<(),Vec<Box<Error>>> {
			
			use vm::bytecode::compiler;
			
			let frame = try!( compiler::compile_script( source ) );
			let fiber = box Fiber::new( frame );
			self.schedule_fiber( fiber );
			Ok( () )
		}
		
		pub fn schedule_repl( &mut self, repl_state: &mut repl::State, source: &str ) -> Result<(),Vec<Box<Error>>> {
			
			use vm::bytecode::compiler;
			
			let frame = try!( compiler::compile_repl( repl_state, source ) );
			let fiber = box Fiber::new( frame );
			self.schedule_fiber( fiber );
			Ok( () )
		}
		
		pub fn to_string( &mut self, value: Value ) -> Result<String,()> {
			
			// todo! maybe use a separate uv_loop?
			
			use vm::bytecode::code::Code;
			use vm::bytecode::opcode;
			use vm::run::frame::Frame;
			
			let mut result = box String::new();
			let result_ptr = &mut *result as *mut String;
			
			let mut code = box Code::new();
			code.n_local_variables = 1;
			code.opcodes = vec!(
				opcode::LoadLocal( 0 ),
				opcode::ToString,
				opcode::Return,
			);
			
			let frame = Frame::new_rust_invoke(
				code,
				vec!( value ),
				vec!()
			);
			
			let mut fiber = box Fiber::new( frame );
			fiber.on_return = Some( proc( to_string_value: Value ) {
				match to_string_value {
					::lang::value::String( s ) => {
						unsafe { *result_ptr = s.borrow().to_string(); }
					}
					_ => { unreachable!(); }
				}
			} );
			
			self.schedule_fiber( fiber );
			self.run();
			
			Ok( *result )
		}
	}
	
	#[unsafe_destructor]
	impl Drop for VirtualMachine {
		fn drop( &mut self ) {
			unsafe { uvll::uv_loop_delete( self.uv_loop ); }
		}
	}
