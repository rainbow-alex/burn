use std::mem;
use std::ptr;

pub struct Raw<T> {
	pub ptr: *T,
}

	impl<T> Raw<T> {
		
		pub fn new( t: &T ) -> Raw<T> {
			Raw { ptr: t as *T }
		}
		
		#[inline(always)]
		pub fn null() -> Raw<T> {
			Raw { ptr: ptr::null() }
		}
		
		#[inline(always)]
		pub fn get( &self ) -> &mut T {
			unsafe { mem::transmute( self.ptr ) }
		}
		
		#[inline(always)]
		pub fn is_null( &self ) -> bool {
			self.ptr == ptr::null()
		}
	}
	
	impl<T> Clone for Raw<T> {
		fn clone( &self ) -> Raw<T> {
			Raw { ptr: self.ptr }
		}
	}
	
	impl<T> Eq for Raw<T> {
		fn eq( &self, other: &Raw<T> ) -> bool {
			self.ptr == other.ptr
		}
	}
	
	impl<T> TotalEq for Raw<T> {}
