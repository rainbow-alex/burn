use lang::identifier::Identifier;
use lang::value;
use builtin::burn;
use builtin::burn::errors::create_type_error;
use mem::rc::Rc;
use vm::run::rust;

pub fn is_truthy( value: &value::Value ) -> bool {
	match *value {
		
		value::Nothing => false,
		value::Boolean( b ) => b,
		value::Integer( i ) => i != 0,
		value::Float( f ) => f != 0f64,
		value::String( ref s ) => s.get().len() > 0,
		
		value::Function(..)
		| value::TypeUnion(..)
		| value::TypeIntersection(..)
		| value::Module(..)
		=> true,
		
		value::StaticSpecial(..) => true,
		value::RcSpecial( ref r ) => r.get().get().is_truthy(),
	}
}

pub fn repr( value: &value::Value ) -> String {
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
		value::RcSpecial( ref r ) => r.get().get().repr(),
	}
}

pub fn to_string( value: &value::Value ) -> rust::Result {
	match *value {
		
		value::Nothing => rust::Ok( value::String( Rc::new( "nothing".into_string() ) ) ),
		value::Boolean( b ) => rust::Ok( value::String( Rc::new( ( if b { "true" } else { "false" } ).into_string() ) ) ),
		value::Integer( i ) => rust::Ok( value::String( Rc::new( format!( "{}", i ) ) ) ),
		value::Float( f ) => rust::Ok( value::String( Rc::new( format!( "{}", f ) ) ) ),
		value::String( ref s ) => rust::Ok( value::String( Rc::new( s.get().clone() ) ) ),
		
		value::StaticSpecial( special ) => rust::Ok( value::String( Rc::new( special.repr() ) ) ),
		value::RcSpecial( ref r ) => rust::Ok( value::String( Rc::new( r.get().get().to_string() ) ) ),
		
		_ => { rust::Ok( value::String( Rc::new( value.repr() ) ) ) }
	}
}

pub fn add( left: &value::Value, right: &value::Value ) -> rust::Result {
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
	
	return rust::Throw( create_type_error( format!( "Can't add {} and {}", left.repr(), right.repr() ) ) );
}

pub fn subtract( left: &value::Value, right: &value::Value ) -> rust::Result {
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
	
	return rust::Throw( create_type_error( format!( "Can't subtract {} and {}", left.repr(), right.repr() ) ) );
}

pub fn multiply( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw( create_type_error( format!( "Can't multiply {} and {}", left.repr(), right.repr() ) ) );
}

pub fn divide( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw( create_type_error( format!( "Can't divide {} and {}", left.repr(), right.repr() ) ) );
}

pub fn union( left: value::Value, right: value::Value ) -> rust::Result {
	
	if ! burn::types::is_type( &left ) {
		return rust::Throw( create_type_error( format!( "Can't create type union: {} is not a type", left.repr() ) ) );
	}
	
	if ! burn::types::is_type( &right ) {
		return rust::Throw( create_type_error( format!( "Can't create type union: {} is not a type", right.repr() ) ) );
	}
	
	rust::Ok( value::TypeUnion( Rc::new( ::lang::type_::TypeUnion::new( left, right ) ) ) )
}

// todo! rust result
pub fn is( value: &value::Value, type_: &value::Value ) -> Result<bool,value::Value> {
	match *type_ {
		
		value::TypeUnion( ref r ) => {
			return match is( value, &r.get().left ) {
				Ok( true ) => Ok( true ),
				Ok( false ) => is( value, &r.get().right ),
				Err( e ) => Err( e ),
			}
		}
		
		value::StaticSpecial( special ) => {
			if special.is_type() {
				return Ok( special.type_test( value ) )
			}
		}
		
		value::RcSpecial( ref r ) => {
			if r.get().get().is_type() {
				return Ok( r.get().get().type_test( value ) )
			}
		}
		
		_ => {}
	}
	
	return Err(
		create_type_error( format!( "{} is not a type", type_.repr() ) )
	);
}

pub fn eq( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn neq( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn lt( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn gt( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn lt_eq( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn gt_eq( left: &value::Value, right: &value::Value ) -> rust::Result {
	return rust::Throw(
		create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) )
	); 
}

pub fn get_property( accessed: &value::Value, name: Identifier ) -> rust::Result {
	(accessed); (name);
	not_implemented!();
}

pub fn set_property( accessed: &value::Value, name: Identifier, value: &value::Value ) -> rust::Result {
	(accessed); (name); (value);
	not_implemented!();
}
