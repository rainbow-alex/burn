use lang::value;
use lang::module::Module;
use lang::special;
use lang::special::{StaticSpecialDef, StaticSpecial, Special, RefCountedSpecial};
use mem::rc::RefCounted;

pub fn create_module() -> Module {
	let mut errors = Module::new();
	errors.add( "TypeError", value::StaticSpecial( StaticSpecial::new( &TypeError ) ) );
	errors.add( "ArgumentError", value::StaticSpecial( StaticSpecial::new( &ArgumentError ) ) );
	errors.lock();
	errors
}



static TypeError: StaticSpecialDef = StaticSpecialDef {
	repr: "TypeError",
	has_method: special::static_has_no_methods,
	type_test: is_type_error,
};

fn is_type_error( value: &value::Value ) -> bool {
	match *value {
		value::RcSpecial( ref r ) => r.borrow().is::<TypeError>(),
		_ => false,
	}
}

struct TypeError {
	message: String,
}

	impl Special for TypeError {
		fn repr( &self ) -> String { "<TypeError>".into_string() }
		fn to_string( &self ) -> String { format!( "TypeError: {}", self.message ) }
		fn is_throwable( &self ) -> bool { true }
	}
	
	impl RefCounted for TypeError {}
	impl RefCountedSpecial for TypeError {}

pub fn create_type_error( message: String ) -> value::Value {
	special::create_rc_value( TypeError { message: message } )
}



static ArgumentError: StaticSpecialDef = StaticSpecialDef {
	repr: "ArgumentError",
	has_method: special::static_has_no_methods,
	type_test: is_argument_error,
};

fn is_argument_error( value: &value::Value ) -> bool {
	match *value {
		value::RcSpecial( ref r ) => r.borrow().is::<ArgumentError>(),
		_ => false,
	}
}

struct ArgumentError {
	message: String,
}

	impl Special for ArgumentError {
		fn repr( &self ) -> String { "<ArgumentError>".into_string() }
		fn to_string( &self ) -> String { format!( "ArgumentError: {}", self.message ) }
		fn is_throwable( &self ) -> bool { true }
	}
	
	impl RefCounted for ArgumentError {}
