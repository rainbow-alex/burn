use lang::module::Module;

pub mod implicit;
pub mod errors;
pub mod types;

pub fn create_module() -> Module {
	let mut burn = Module::new();
	burn.add_module( "implicit", box implicit::create_module() );
	burn.add_module( "errors", box errors::create_module() );
	burn.add_module( "types", box types::create_module() );
	burn.lock();
	burn
}
