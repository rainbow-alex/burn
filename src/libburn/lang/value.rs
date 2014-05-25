use std::mem;
use std::ptr;
use lang::function::Function;
use mem::gc::Gc;
use mem::rc::Rc;
use mem::raw::Raw;
use lang::string::String;
use lang::type_::{TypeUnion, TypeIntersection};
use lang::module::Module;
use lang::special::{StaticSpecial, RcSpecial};
use builtin::intrinsic::operations;

#[deriving(Clone)]
pub enum Value {
	
	Nothing,
	Boolean( bool ),
	Integer( i64 ),
	Float( f64 ),
	String( Rc<String> ),
	
	Function( Gc<Function> ),
	TypeUnion( Rc<TypeUnion> ),
	TypeIntersection( Rc<TypeIntersection> ),
	Module( Raw<Module> ),
	
	StaticSpecial( StaticSpecial ),
	RcSpecial( Rc<RcSpecial> ),
}

	impl Value {
		#[inline]
		pub fn repr( &self ) -> StrBuf { operations::repr( self ) }
		#[inline]
		pub fn to_string( &self ) -> StrBuf { operations::to_string( self ) }
	}

struct SharedValueContainer {
	rc: uint,
	value: Value,
}

#[unsafe_no_drop_flag]
pub struct SharedValue {
	ptr: *SharedValueContainer,
}

	impl SharedValue {
		
		pub fn new( value: Value ) -> SharedValue {
			let container = box SharedValueContainer { rc: 1, value: value };
			SharedValue { ptr: unsafe { mem::transmute::<_,*SharedValueContainer>( container ) } }
		}
		
		fn get_container( &self ) -> &mut SharedValueContainer {
			unsafe { mem::transmute( self.ptr ) }
		}
		
		pub fn get( &self ) -> &mut Value {
			unsafe { mem::transmute( &(*self.ptr).value ) }
		}
	}
	
	impl Clone for SharedValue {
		fn clone( &self ) -> SharedValue {
			if self.ptr != ptr::null() {
				self.get_container().rc += 1;
			}
			SharedValue { ptr: self.ptr }
		}
	}
	
	#[unsafe_destructor]
	impl Drop for SharedValue {
		
		fn drop( &mut self ) {
			if ! self.ptr.is_null() {
				
				self.get_container().rc -= 1;
				
				if self.get_container().rc == 0 {
					drop( unsafe { mem::transmute::<_,Box<SharedValueContainer>>( self.ptr ) } );
				}
				
				self.ptr = ptr::null();
			}
		}
	}

#[cfg(test)]
mod test {
	
	use lang::value;
	use lang::value::SharedValue;
	
	#[test]
	fn test_shared() {
		let shared1 = SharedValue::new( value::Nothing );
		let shared2 = shared1.clone();
		
		assert!( match *shared1.get() { value::Nothing => true, _ => false } );
		assert!( match *shared2.get() { value::Nothing => true, _ => false } );
		
		*shared1.get() = value::Integer( 3 );
		
		assert!( match *shared1.get() { value::Integer( 3 ) => true, _ => false } );
		assert!( match *shared2.get() { value::Integer( 3 ) => true, _ => false } );
	}
}
