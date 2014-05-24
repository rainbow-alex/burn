use lang::value;
use lang::string::String;
use lang::special;
use lang::special::{StaticSpecialDef, Special, RcSpecial};

pub static TypeError: StaticSpecialDef = StaticSpecialDef {
	repr: "TypeError",
	has_method: special::static_has_no_methods,
	type_test: is_type_error,
};

fn is_type_error( value: &value::Value ) -> bool {
	match *value {
		value::RcSpecial( ref r ) => r.get().is::<TypeError>(),
		_ => false,
	}
}

struct TypeError {
	message: String,
}

	impl Special for TypeError {
		fn repr( &self ) -> StrBuf { "<TypeError>".into_owned() }
		fn to_string( &self ) -> StrBuf { format!( "TypeError: {}", self.message ) }
		fn is_throwable( &self ) -> bool { true }
	}
	
	impl RcSpecial for TypeError {}

pub fn create_type_error( message: StrBuf ) -> value::Value {
	special::create_rc_value( TypeError { message: String::new( message ) } )
}



pub static ArgumentError: StaticSpecialDef = StaticSpecialDef {
	repr: "ArgumentError",
	has_method: special::static_has_no_methods,
	type_test: is_argument_error,
};

fn is_argument_error( value: &value::Value ) -> bool {
	match *value {
		value::RcSpecial( ref r ) => r.get().is::<ArgumentError>(),
		_ => false,
	}
}

struct ArgumentError {
	message: String,
}

	impl Special for ArgumentError {
		fn repr( &self ) -> StrBuf { "<ArgumentError>".into_owned() }
		fn to_string( &self ) -> StrBuf { format!( "ArgumentError: {}", self.message ) }
		fn is_throwable( &self ) -> bool { true }
	}
	
	impl RcSpecial for ArgumentError {}
