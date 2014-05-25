use lang::value;
use lang::special;
use lang::special::{StaticSpecialDef, StaticSpecial};
use lang::module::Module;

pub fn create_module() -> Module {
	let mut types = Module::new();
	types.add( "Throwable", value::StaticSpecial( StaticSpecial::new( &Throwable ) ) );
	types.lock();
	types
}



static Throwable: StaticSpecialDef = StaticSpecialDef {
	repr: "Throwable",
	has_method: special::static_has_no_methods,
	type_test: is_throwable,
};

pub fn is_throwable( value: &value::Value ) -> bool {
	match *value {
		value::RcSpecial( ref r ) => r.get().get().is_throwable(),
		_ => false,
	}
}
