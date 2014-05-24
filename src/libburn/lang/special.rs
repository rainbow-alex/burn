use lang::value;
use lang::identifier::Identifier;
use mem::rc::{Rc, RcHeader, RefCounted};

pub trait Special {
	fn repr( &self ) -> StrBuf;
	fn to_string( &self ) -> StrBuf { self.repr() }
	fn is_truthy( &self ) -> bool { true }
	fn is_type( &self ) -> bool { false }
	fn type_test( &self, &value::Value ) -> bool { fail!() }
	fn is_throwable( &self ) -> bool { false }
}

pub trait RcSpecial : Special {}

pub struct RcSpecialWrapper {
	rc: RcHeader,
	type_id: ::core::intrinsics::TypeId,
	special: Box<RcSpecial>,
}

	impl RcSpecialWrapper {
		
		pub fn is<T:'static>( &self ) -> bool {
			unsafe { ::core::intrinsics::type_id::<T>() == self.type_id }
		}
		
		#[inline]
		pub fn repr( &self ) -> StrBuf { self.special.repr() }
		#[inline]
		pub fn to_string( &self ) -> StrBuf { self.special.to_string() }
		#[inline]
		pub fn is_truthy( &self ) -> bool { self.special.is_truthy() }
		#[inline]
		pub fn is_type( &self ) -> bool { self.special.is_type() }
		#[inline]
		pub fn type_test( &self, value: &value::Value ) -> bool { self.special.type_test( value ) }
		#[inline]
		pub fn is_throwable( &self ) -> bool { self.special.is_throwable() }
	}
	
	impl RefCounted for RcSpecialWrapper {
		fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
			&mut self.rc
		}
	}

pub fn create_rc_value<T:'static+RcSpecial>( special: T ) -> value::Value {
	value::RcSpecial( Rc::new( box RcSpecialWrapper {
		rc: RcHeader::new(),
		type_id: unsafe { ::core::intrinsics::type_id::<T>() },
		special: box special as Box<RcSpecial>,
	} ) )
}



pub fn static_has_no_methods( _: Identifier ) -> bool { false }
pub fn static_not_a_type( _: &value::Value ) -> bool { fail!() }

pub struct StaticSpecialDef {
	pub repr: &'static str,
	pub has_method: fn ( Identifier ) -> bool,
	pub type_test: fn ( &value::Value ) -> bool,
}

#[deriving(Clone)]
pub struct StaticSpecial {
	pub def: &'static StaticSpecialDef,
}

	impl StaticSpecial {
		
		pub fn new( def: &'static StaticSpecialDef ) -> StaticSpecial {
			StaticSpecial { def: def }
		}
		
		#[inline]
		pub fn repr( self ) -> StrBuf { self.def.repr.to_owned() }
		#[inline]
		pub fn is_truthy( self ) -> bool { true }
		#[inline]
		pub fn is_type( self ) -> bool { &self.def.type_test as *_ != &static_not_a_type as *_ }
		#[inline]
		pub fn type_test( self, value: &value::Value ) -> bool { ( self.def.type_test )( value ) }
		#[inline]
		pub fn is_throwable( self ) -> bool { false }
	}
