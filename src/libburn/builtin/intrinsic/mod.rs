use lang::module::Module;
use lang::identifier::id;
use lang::value;
use lang::special::StaticSpecial;

pub mod types;
pub mod operations;
pub mod errors;

pub fn create_module() -> Module {
	
	let mut intrinsic = Module::new();
	
	intrinsic.set( id( "Boolean" ), value::StaticSpecial( StaticSpecial::new( &types::Boolean ) ) );
	intrinsic.set( id( "Integer" ), value::StaticSpecial( StaticSpecial::new( &types::Integer ) ) );
	intrinsic.set( id( "Float" ), value::StaticSpecial( StaticSpecial::new( &types::Float ) ) );
	intrinsic.set( id( "Number" ), value::StaticSpecial( StaticSpecial::new( &types::Number ) ) );
	intrinsic.set( id( "String" ), value::StaticSpecial( StaticSpecial::new( &types::String ) ) );
	intrinsic.set( id( "Type" ), value::StaticSpecial( StaticSpecial::new( &types::Type ) ) );
	intrinsic.set( id( "Throwable" ), value::StaticSpecial( StaticSpecial::new( &types::Throwable ) ) );
	
	intrinsic.set( id( "TypeError" ), value::StaticSpecial( StaticSpecial::new( &errors::TypeError ) ) );
	intrinsic.set( id( "ArgumentError" ), value::StaticSpecial( StaticSpecial::new( &errors::ArgumentError ) ) );
	
	intrinsic
}
