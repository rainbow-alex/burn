use lang::value;
use lang::special;
use lang::special::StaticSpecialDef;

pub static Boolean: StaticSpecialDef = StaticSpecialDef {
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

pub static Integer: StaticSpecialDef = StaticSpecialDef {
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

pub static Float: StaticSpecialDef = StaticSpecialDef {
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

pub static Number: StaticSpecialDef = StaticSpecialDef {
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

pub static String: StaticSpecialDef = StaticSpecialDef {
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

pub static Type: StaticSpecialDef = StaticSpecialDef {
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

pub static Throwable: StaticSpecialDef = StaticSpecialDef {
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
