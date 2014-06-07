use lang::identifier::Identifier;
use lang::value;
use lang::value::Value;
use builtin::burn;
use builtin::burn::errors::create_type_error;
use mem::rc::Rc;
use vm::run::rust;

pub fn is_truthy( value: &Value ) -> bool {
	match *value {
		
		value::Nothing => false,
		value::Boolean( b ) => b,
		value::Integer( i ) => i != 0,
		value::Float( f ) => f != 0f64,
		value::String( ref s ) => s.len() > 0,
		
		value::Function(..)
		| value::TypeUnion(..)
		| value::TypeIntersection(..)
		| value::Module(..)
		=> true,
		
		value::StaticSpecial(..) => true,
		value::RcSpecial( ref r ) => r.is_truthy(),
	}
}

pub fn repr( value: &Value ) -> String {
	match *value {
		
		value::Nothing => "<Nothing>".to_string(),
		value::Boolean(..) => "<Boolean>".to_string(),
		value::Integer(..) => "<Integer>".to_string(),
		value::Float(..) => "<Float>".to_string(),
		value::String(..) => "<String>".to_string(),
		
		value::Function(..) => "<Function>".to_string(),
		value::TypeUnion(..) | value::TypeIntersection(..) => "<Type>".to_string(),
		value::Module(..) => "<Module>".to_string(),
		
		value::StaticSpecial( special ) => special.repr(),
		value::RcSpecial( ref r ) => r.repr(),
	}
}

pub fn to_string( value: &Value ) -> rust::Result {
	rust::Ok( value::String(
		match *value {
			
			value::Nothing => Rc::new( "nothing".into_string() ),
			value::Boolean( true ) => Rc::new( "true".into_string() ),
			value::Boolean( false ) => Rc::new( "false".into_string() ),
			value::Integer( i ) => Rc::new( format!( "{}", i ) ),
			value::Float( f ) => Rc::new( format!( "{}", f ) ),
			value::String( ref s ) => s.clone(),
			
			value::StaticSpecial( special ) => Rc::new( special.repr() ),
			value::RcSpecial( ref r ) => Rc::new( r.to_string() ),
			
			_ => { Rc::new( repr( value ) ) }
		}
	) )
}

pub fn add( left: &Value, right: &Value ) -> rust::Result {
	match *left {
		
		value::Integer( l ) => {
			match *right {
				value::Integer( r ) => { return rust::Ok( value::Integer( l + r ) ); }
				value::Float( r ) => { return rust::Ok( value::Float( l as f64 + r ) ); }
				_ => {}
			}
		}
		
		value::Float( l ) => {
			match *right {
				value::Integer( r ) => { return rust::Ok( value::Float( l + r as f64 ) ); }
				value::Float( r ) => { return rust::Ok( value::Float( l + r ) ); }
				_ => {}
			}
		}
		
		_ => {}
	}
	
	return rust::Throw(
		create_type_error( format!( "Can't add {} and {}", repr( left ), repr( right ) ) )
	);
}

pub fn subtract( left: &Value, right: &Value ) -> rust::Result {
	match *left {
		
		value::Integer( l ) => {
			match *right {
				value::Integer( r ) => { return rust::Ok( value::Integer( l - r ) ); }
				value::Float( r ) => { return rust::Ok( value::Float( l as f64 - r ) ); }
				_ => {}
			}
		}
		
		value::Float( l ) => {
			match *right {
				value::Integer( r ) => { return rust::Ok( value::Float( l - r as f64 ) ); }
				value::Float( r ) => { return rust::Ok( value::Float( l - r ) ); }
				_ => {}
			}
		}
		
		_ => {}
	}
	
	return rust::Throw(
		create_type_error( format!( "Can't subtract {} and {}", repr( left ), repr( right ) ) )
	);
}

pub fn multiply( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't multiply {} and {}", repr( left ), repr( right ) ) )
	);
}

pub fn divide( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't divide {} and {}", repr( left ), repr( right ) ) )
	);
}

pub fn union( left: Value, right: Value ) -> rust::Result {
	
	if ! burn::types::is_type( &left ) {
		return rust::Throw(
			create_type_error( format!( "Can't create type union: {} is not a type", repr( &left ) ) )
		);
	}
	
	if ! burn::types::is_type( &right ) {
		return rust::Throw(
			create_type_error( format!( "Can't create type union: {} is not a type", repr( &right ) ) )
		);
	}
	
	rust::Ok( value::TypeUnion( Rc::new( ::lang::type_::TypeUnion::new( left, right ) ) ) )
}

pub fn is( value: &Value, type_: &Value ) -> rust::Result {
	match *type_ {
		
		value::TypeUnion( ref r ) => {
			return match is( value, &r.left ) {
				rust::Ok( value::Boolean( false ) ) => is( value, &r.right ),
				other_result @ _ => other_result,
			}
		}
		
		value::StaticSpecial( special ) => {
			if special.is_type() {
				return rust::Ok( value::Boolean( special.type_test( value ) ) )
			}
		}
		
		value::RcSpecial( ref r ) => {
			if r.is_type() {
				return rust::Ok( value::Boolean( r.type_test( value ) ) )
			}
		}
		
		_ => {}
	}
	
	return rust::Throw(
		create_type_error( format!( "{} is not a type", repr( type_ ) ) )
	);
}

pub fn eq( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn neq( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn lt( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn gt( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn lt_eq( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn gt_eq( left: &Value, right: &Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", repr( left ), repr( right ) ) )
	); 
}

pub fn get_property( accessed: &Value, name: Identifier ) -> rust::Result {
	(accessed); (name);
	unimplemented!();
}

pub fn set_property( accessed: &Value, name: Identifier, value: &Value ) -> rust::Result {
	(accessed); (name); (value);
	unimplemented!();
}
