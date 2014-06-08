use rustuv::uvll;
use libc::c_void;
use mem::gc::GarbageCollectedManager;
use mem::raw::Raw;
use mem::rc::Rc;
use lang::origin;
use lang::origin::Origin;
use lang::function::Function;
use lang::module::Module;
use lang::value::Value;
use vm::run::fiber::Fiber;
use vm::error::{Error, UncaughtThrowableHandler};
use vm::repl;

/// The burn VM manages memory, schedules events and runs code.
///
/// Memory is garbage-collected. Collection is *not* deterministic.
/// Currently the VM uses refcounting, combined with a mark-and-sweep algorithm to detect cycles.
/// Some language features (e.g. module properties not being re-assignable)
/// allow for interesting optimizations to this simple algorithm.
///
/// Code is executed in light-weight threads called fibers.
/// A fiber can suspend (`yield`) itself whenever it chooses;
/// a suspended fiber can be scheduled, and continues executing where it yielded.
/// Fibers are run in parallel, but never concurrently, in the rust task that calls `run`.
/// This allows for safely shared memory, and concurrent IO without callbacks.
pub struct VirtualMachine {
	#[doc(hidden)]
	pub functions: GarbageCollectedManager<Function>,
	#[doc(hidden)]
	pub import_paths: Vec<Path>,
	#[doc(hidden)]
	pub module_root: Box<Module>,
	#[doc(hidden)]
	pub implicit: Raw<Module>,
	#[doc(hidden)]
	uv_loop: *c_void,
	#[doc(hidden)]
	pub uncaught_throwable_handlers: Vec<Box<UncaughtThrowableHandler>>,
}

	impl VirtualMachine {
		
		/// Create a new virtual machine.
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
		
		/// Register an `UncaughtThrowableHandler`.
		pub fn on_uncaught_throwable( &mut self, handler: Box<UncaughtThrowableHandler> ) {
			self.uncaught_throwable_handlers.push( handler );
		}
		
		/// Run scheduled events until the queue is empty.
		pub fn run( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_ONCE ); }
		}
		
		/// Run scheduled events forever. Waits for new events whenever the queue is empty.
		pub fn run_loop( &mut self ) {
			unsafe { uvll::uv_run( self.uv_loop, uvll::RUN_DEFAULT ); }
		}
		
		/// Schedule a rust procedure to be executed.
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
		
		fn schedule_fiber( &mut self, fiber: Box<Fiber> ) {
			self.schedule( proc( vm ) {
				use vm::run::cpu;
				cpu::run( vm, fiber );
			} );
		}
		
		/// Compile some source code and schedule it for execution.
		/// If provided, root-level variables will be persisted in `repl_state`.
		///
		/// Any compilation errors are returned immediately.
		pub fn schedule_source( &mut self, origin: Box<Origin>, repl_state: Option<&mut repl::State>, source_code: &str ) -> Result<(),Vec<Box<Error>>> {
			
			use vm::bytecode::compiler;
			
			let origin = Rc::new( origin );
			let frame = try!( compiler::compile( origin, repl_state, source_code ) );
			let fiber = box Fiber::new( frame );
			self.schedule_fiber( fiber );
			Ok( () )
		}
		
		/// Convert a value into a `String` by creating and running a fiber
		/// to run the necessary burn code.
		/// This method blocks the current task until the conversion is complete.
		///
		/// * FIXME: return uncaught throwable, if any
		/// * FIXME: use a separate uv_loop
		pub fn to_string( &mut self, value: Value ) -> Result<String,()> {
			
			use vm::bytecode::code::Code;
			use vm::bytecode::opcode;
			use vm::run::frame;
			
			let mut result = box String::new();
			let result_ptr = &mut *result as *mut String;
			
			let mut code = box Code::new();
			code.n_local_variables = 1;
			code.opcodes = vec!(
				opcode::LoadLocal( 0 ),
				opcode::ToString,
				opcode::Return,
			);
			
			let origin = box origin::Rust { name: "to_string".to_string() } as Box<Origin>;
			
			let frame = frame::BurnRootFrame {
				origin: Rc::new( origin ),
				code: code,
				context: frame::BurnContext::new( vec!( value ), vec!() ),
			};
			
			let mut fiber = box Fiber::new( frame );
			fiber.on_return = Some( proc( to_string_value: Value ) {
				match to_string_value {
					::lang::value::String( mut s ) => {
						unsafe { *result_ptr = s.to_string(); }
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
