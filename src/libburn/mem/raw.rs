use std::mem;
use std::ptr;

pub struct Raw<T> {
	pub ptr: *mut T,
}

	impl<T> Raw<T> {
		
		pub fn new( t: &T ) -> Raw<T> {
			Raw { ptr: unsafe { mem::transmute( t ) } }
		}
		
		pub fn null() -> Raw<T> {
			Raw { ptr: ptr::mut_null() }
		}
		
		pub unsafe fn get_box( &self ) -> Box<T> {
			mem::transmute( self.ptr )
		}
		
		pub fn is_null( &self ) -> bool {
			self.ptr == ptr::mut_null()
		}
	}
	
	impl<T> Deref<T> for Raw<T> {
		fn deref<'l>( &'l self ) -> &'l T {
			unsafe { &*self.ptr }
		}
	}
	
	impl<T> DerefMut<T> for Raw<T> {
		fn deref_mut<'l>( &'l mut self ) -> &'l mut T {
			unsafe { &mut*self.ptr }
		}
	}
	
	impl<T> Clone for Raw<T> {
		fn clone( &self ) -> Raw<T> {
			Raw { ptr: self.ptr }
		}
	}
	
	impl<T> PartialEq for Raw<T> {
		fn eq( &self, other: &Raw<T> ) -> bool {
			self.ptr == other.ptr
		}
	}
	
	impl<T> Eq for Raw<T> {}
