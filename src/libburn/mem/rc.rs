use std::mem;
use std::ptr;

#[unsafe_no_drop_flag]
pub struct Rc<T> {
	ptr: *mut RcWrapper<T>,
}

	impl<T:RefCounted> Rc<T> {
		
		pub fn new( thing: T ) -> Rc<T> {
			let rc_wrapper = box RcWrapper {
				rc: 1,
				value: thing,
			};
			Rc { ptr: unsafe { mem::transmute::<_,*mut RcWrapper<T>>( rc_wrapper ) } }
		}
		
		#[inline(always)]
		pub fn get( &self ) -> &mut T {
			unsafe { mem::transmute( &(*self.ptr).value ) }
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
	value: T,
}

pub trait RefCounted {}

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
				unsafe {
					*self.dropped = true;
				}
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

#[cfg(test)]
mod bench {
	
	use std::mem;
	use test::Bencher;
	use mem::rc::{Rc, RefCounted};
	
	struct Thing {
		a: int,
	}
	
		impl RefCounted for Thing {}
	
	#[bench]
	fn bench_raw_deref( bench: &mut Bencher ) {
		let thing = Thing {
			a: 3,
		};
		let r: *Thing = unsafe { mem::transmute( thing ) };
		
		bench.iter( || {
			assert!( unsafe { (*r).a } == 3 );
		} );
	}
	
	#[bench]
	fn bench_rc_get( bench: &mut Bencher ) {
		let thing = Thing {
			a: 3,
		};
		let r = Rc::new( thing );
		
		bench.iter( || {
			assert!( r.get().a == 3 );
		} );
	}
}
