use lang::module::Module;
use lang::value;
use lang::special;
use lang::special::{StaticSpecialDef, StaticSpecial};

pub fn create_module() -> Module {
	let mut implicit = Module::new();
	implicit.add( "Boolean", value::StaticSpecial( StaticSpecial::new( &Boolean ) ) );
	implicit.add( "Integer", value::StaticSpecial( StaticSpecial::new( &Integer ) ) );
	implicit.add( "Float", value::StaticSpecial( StaticSpecial::new( &Float ) ) );
	implicit.add( "Number", value::StaticSpecial( StaticSpecial::new( &Number ) ) );
	implicit.add( "String", value::StaticSpecial( StaticSpecial::new( &String ) ) );
	implicit.add( "Type", value::StaticSpecial( StaticSpecial::new( &Type ) ) );
	implicit.lock();
	implicit
}



static Boolean: StaticSpecialDef = StaticSpecialDef {
	repr: "Boolean",
	has_method: special::static_has_no_methods,
	type_test: is_boolean,
};

fn is_boolean( value: &value::Value ) -> bool {
	match *value {
		value::Boolean(..) => true,
		_ => false,
	}
}



static Integer: StaticSpecialDef = StaticSpecialDef {
	repr: "Integer",
	has_method: special::static_has_no_methods,
	type_test: is_integer,
};

fn is_integer( value: &value::Value ) -> bool {
	match *value {
		value::Integer(..) => true,
		_ => false,
	}
}



static Float: StaticSpecialDef = StaticSpecialDef {
	repr: "Float",
	has_method: special::static_has_no_methods,
	type_test: is_float,
};

fn is_float( value: &value::Value ) -> bool {
	match *value {
		value::Float(..) => true,
		_ => false,
	}
}



static Number: StaticSpecialDef = StaticSpecialDef {
	repr: "Number",
	has_method: special::static_has_no_methods,
	type_test: is_number,
};

fn is_number( value: &value::Value ) -> bool {
	match *value {
		value::Integer(..) | value::Float(..) => true,
		_ => false,
	}
}



static String: StaticSpecialDef = StaticSpecialDef {
	repr: "String",
	has_method: special::static_has_no_methods,
	type_test: is_string,
};

fn is_string( value: &value::Value ) -> bool {
	match *value {
		value::String(..) => true,
		_ => false,
	}
}



static Type: StaticSpecialDef = StaticSpecialDef {
	repr: "Type",
	has_method: special::static_has_no_methods,
	type_test: is_type,
};

pub fn is_type( value: &value::Value ) -> bool {
	match *value {
		value::TypeUnion(..) => true,
		value::TypeIntersection(..) => true,
		value::StaticSpecial( special ) => special.is_type(),
		value::RcSpecial( ref r ) => r.get().get().is_type(),
		_ => false,
	}
}
