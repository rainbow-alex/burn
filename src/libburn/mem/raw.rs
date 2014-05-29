use std::mem;
use std::ptr;
use std::fmt;

pub struct Raw<T> {
	pub ptr: *mut T,
}

	impl<T> Raw<T> {
		
		#[inline(always)]
		pub fn new( t: &T ) -> Raw<T> {
			Raw { ptr: unsafe { mem::transmute( t ) } }
		}
		
		#[inline(always)]
		pub fn null() -> Raw<T> {
			Raw { ptr: ptr::mut_null() }
		}
		
		#[inline(always)]
		pub fn get( &self ) -> &mut T {
			unsafe { &mut *self.ptr }
		}
		
		#[inline(always)]
		pub unsafe fn get_box( &self ) -> Box<T> {
			mem::transmute( self.ptr )
		}
		
		#[inline(always)]
		pub fn is_null( &self ) -> bool {
			self.ptr == ptr::mut_null()
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
	
	impl<T> fmt::Show for Raw<T> {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			write!( f, "{}", self.ptr )
		}
	}
