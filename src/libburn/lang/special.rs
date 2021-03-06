use lang::value;
use lang::identifier::Identifier;
use mem::rc::{Rc, RefCounted};

// todo! rust results

pub trait Special {
	fn repr( &self ) -> String;
	fn to_string( &self ) -> String { self.repr() }
	fn is_truthy( &self ) -> bool { true }
	fn is_type( &self ) -> bool { false }
	fn type_test( &self, &value::Value ) -> bool { unreachable!() }
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
	}
	
	impl Deref<Box<RefCountedSpecial>> for RcSpecial {
		fn deref<'l>( &'l self ) -> &'l Box<RefCountedSpecial> {
			& self.special
		}
	}
	
	impl DerefMut<Box<RefCountedSpecial>> for RcSpecial {
		fn deref_mut<'l>( &'l mut self ) -> &'l mut Box<RefCountedSpecial> {
			&mut self.special
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
pub fn static_not_a_type( _: &value::Value ) -> bool { unreachable!() }

pub struct StaticSpecialDef {
	pub repr: &'static str,
	pub has_method: fn ( Identifier ) -> bool,
	pub type_test: fn ( &value::Value ) -> bool,
}

#[deriving(Clone)]
pub struct StaticSpecial {
	def: &'static StaticSpecialDef,
}

	impl StaticSpecial {
		
		pub fn new( def: &'static StaticSpecialDef ) -> StaticSpecial {
			StaticSpecial { def: def }
		}
		
		pub fn repr( self ) -> String { self.def.repr.to_string() }
		pub fn is_truthy( self ) -> bool { true }
		pub fn is_type( self ) -> bool { &self.def.type_test as *_ != &static_not_a_type as *_ }
		pub fn type_test( self, value: &value::Value ) -> bool { ( self.def.type_test )( value ) }
		pub fn is_throwable( self ) -> bool { false }
	}
