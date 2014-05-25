use lang::value;
use lang::identifier::Identifier;
use mem::rc::{Rc, RefCounted};

pub trait Special {
	fn repr( &self ) -> StrBuf;
	fn to_string( &self ) -> StrBuf { self.repr() }
	fn is_truthy( &self ) -> bool { true }
	fn is_type( &self ) -> bool { false }
	fn type_test( &self, &value::Value ) -> bool { fail!() }
	fn is_throwable( &self ) -> bool { false }
}

pub trait RefCountedSpecial : Special + RefCounted {}

pub struct RcSpecial {
	type_id: ::core::intrinsics::TypeId,
	special: Box<RefCountedSpecial>,
}

	impl RcSpecial {
		
		pub fn is<T:'static>( &self ) -> bool {
			unsafe { ::core::intrinsics::type_id::<T>() == self.type_id }
		}
		
		#[inline(always)]
		pub fn get<'a>( &'a mut self ) -> &'a mut RefCountedSpecial {
			// won't coerce without a tmp var, seems to be a bug
			let tmp: &mut RefCountedSpecial = self.special;
			tmp
		}
	}

	impl RefCounted for RcSpecial {}

pub fn create_rc_value<T:RefCountedSpecial+'static>( special: T ) -> value::Value {
	value::RcSpecial( Rc::new( RcSpecial {
		type_id: unsafe { ::core::intrinsics::type_id::<T>() },
		special: box special,
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
