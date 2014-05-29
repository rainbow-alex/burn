use lang::identifier::Identifier;
use lang::value;
use builtin::burn::implicit;
use builtin::burn::errors::create_type_error;
use mem::rc::Rc;

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

pub fn to_string( value: &value::Value ) -> String {
	match *value {
		
		value::Nothing => "nothing".into_owned(),
		value::Boolean( b ) => ( if b { "true" } else { "false" } ).into_owned(),
		value::Integer( i ) => format!( "{}", i ),
		value::Float( f ) => format!( "{}", f ),
		value::String( ref s ) => s.get().clone(),
		
		value::StaticSpecial( special ) => special.repr(),
		value::RcSpecial( ref r ) => r.get().get().to_string(),
		
		_ => { value.repr() }
	}
}

pub fn add( left: &value::Value, right: &value::Value ) -> Result<value::Value,value::Value> {
	match *left {
		
		value::Integer( l ) => {
			match *right {
				value::Integer( r ) => { return Ok( value::Integer( l + r ) ); }
				value::Float( r ) => { return Ok( value::Float( l as f64 + r ) ); }
				_ => {}
			}
		}
		
		value::Float( l ) => {
			match *right {
				value::Integer( r ) => { return Ok( value::Float( l + r as f64 ) ); }
				value::Float( r ) => { return Ok( value::Float( l + r ) ); }
				_ => {}
			}
		}
		
		_ => {}
	}
	
	return Err( create_type_error( format!( "Can't add {} and {}", left.repr(), right.repr() ) ) );
}

pub fn subtract( left: &value::Value, right: &value::Value ) -> Result<value::Value,value::Value> {
	match *left {
		
		value::Integer( l ) => {
			match *right {
				value::Integer( r ) => { return Ok( value::Integer( l - r ) ); }
				value::Float( r ) => { return Ok( value::Float( l as f64 - r ) ); }
				_ => {}
			}
		}
		
		value::Float( l ) => {
			match *right {
				value::Integer( r ) => { return Ok( value::Float( l - r as f64 ) ); }
				value::Float( r ) => { return Ok( value::Float( l - r ) ); }
				_ => {}
			}
		}
		
		_ => {}
	}
	
	return Err( create_type_error( format!( "Can't subtract {} and {}", left.repr(), right.repr() ) ) );
}

pub fn union( left: value::Value, right: value::Value ) -> Result<value::Value,value::Value> {
	
	if ! implicit::is_type( &left ) {
		return Err( create_type_error( format!( "Can't create type union: {} is not a type", left.repr() ) ) );
	}
	
	if ! implicit::is_type( &right ) {
		return Err( create_type_error( format!( "Can't create type union: {} is not a type", right.repr() ) ) );
	}
	
	Ok( value::TypeUnion( Rc::new( ::lang::type_::TypeUnion::new( left, right ) ) ) )
}

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
	
	return Err( create_type_error(
		format!( "{} is not a type", type_.repr() )
	) )
}

pub fn eq( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	return Err( create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) ) ); 
}

pub fn neq( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	return Err( create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) ) ); 
}

pub fn lt( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	return Err( create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) ) ); 
}

pub fn gt( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	return Err( create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) ) ); 
}

pub fn lt_eq( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	gt( left, right ).and_then( |v| { Ok( !v ) } )
}

pub fn gt_eq( left: &value::Value, right: &value::Value ) -> Result<bool,value::Value> {
	return Err( create_type_error( format!( "Can't compare {} and {}", left.repr(), right.repr() ) ) ); 
}

pub fn get_property( accessed: &value::Value, name: Identifier ) -> Result<value::Value, value::Value> {
	(accessed); (name);
	Ok( value::Nothing ) // TODO
}

pub fn set_property( accessed: &value::Value, name: Identifier, value: &value::Value ) -> Result<(), value::Value> {
	(accessed); (name); (value);
	Ok( () ) // TODO
}
