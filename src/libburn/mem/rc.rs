use std::mem;
use std::ptr;

#[unsafe_no_drop_flag]
pub struct Rc<T> {
	ptr: *mut RcWrapper<T>,
}

	impl<T:RefCounted> Rc<T> {
		
		pub fn new( thing: T ) -> Rc<T> {
			let mut rc_wrapper = box RcWrapper {
				rc: 1,
				value: thing,
			};
			let ptr = &mut *rc_wrapper as *mut RcWrapper<T>;
			unsafe { mem::forget( rc_wrapper ); }
			Rc { ptr: ptr }
		}
	}
	
	impl<T> Deref<T> for Rc<T> {
		fn deref<'l>( &'l self ) -> &'l T {
			unsafe { & (*self.ptr).value }
		}
	}
	
	impl<T> DerefMut<T> for Rc<T> {
		fn deref_mut<'l>( &'l mut self ) -> &'l mut T {
			unsafe { &mut (*self.ptr).value }
		}
	}
	
	impl<T:RefCounted> Clone for Rc<T> {
		fn clone( &self ) -> Rc<T> {
			unsafe { (*self.ptr).rc += 1; }
			Rc { ptr: self.ptr }
		}
	}
	
	#[unsafe_destructor]
	impl<T:RefCounted> Drop for Rc<T> {
		fn drop( &mut self ) {
			unsafe {
				if ! self.ptr.is_null() {
					
					(*self.ptr).rc -= 1;
					if (*self.ptr).rc == 0 {
						drop( mem::transmute::<_,Box<RcWrapper<T>>>( self.ptr ) );
					}
					
					self.ptr = ptr::mut_null();
				}
			}
		}
	}

struct RcWrapper<T> {
	rc: uint,
	// todo! wrap in Unsafe and put markers on this.
	value: T,
}

pub trait RefCounted {}

impl RefCounted for String {}

#[cfg(test)]
mod test {
	
	use mem::rc::{Rc, RefCounted};
	
	struct Thing {
		dropped: *mut bool,
	}
	
		impl Thing {
			
			pub fn new( dropped: *mut bool ) -> Thing {
				Thing { dropped: dropped }
			}
		}
		
		impl RefCounted for Thing {}
		
		impl Drop for Thing {
			fn drop( &mut self ) {
				unsafe { *self.dropped = true; }
			}
		}
	
	#[test]
	fn test_drop() {
		let mut dropped = false;
		let thing = Thing::new( &mut dropped );
		
		let r = Rc::new( thing );
		assert!( ! dropped );
		
		drop( r );
		assert!( dropped );
	}
	
	#[test]
	fn test_clone() {
		let mut dropped = false;
		let thing = Thing::new( &mut dropped );
		
		let r1 = Rc::new( thing );
		let r2 = r1.clone();
		
		drop( r1 );
		assert!( ! dropped );
		
		drop( r2 );
		assert!( dropped );
	}
}
