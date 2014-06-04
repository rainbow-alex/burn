use lang::module::Module;

pub mod errors;
pub mod types;

pub fn create_module() -> Module {
	let mut burn = Module::new();
	
	let types = box types::create_module();
	let errors = box errors::create_module();
	
	let mut implicit = box Module::new();
	implicit.add( "Boolean", types.get( "Boolean" ) );
	implicit.add( "Integer", types.get( "Integer" ) );
	implicit.add( "Float", types.get( "Float" ) );
	implicit.add( "Number", types.get( "Number" ) );
	implicit.add( "String", types.get( "String" ) );
	implicit.add( "Type", types.get( "Type" ) );
	implicit.add( "ArgumentError", errors.get( "ArgumentError" ) );
	implicit.add( "TypeError", errors.get( "TypeError" ) );
	implicit.lock();
	
	burn.add_module( "types", types );
	burn.add_module( "errors", errors );
	burn.add_module( "implicit", implicit );
	
	burn.lock();
	burn
}
