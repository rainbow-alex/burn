use std::mem;
use std::ptr;
use mem::raw::Raw;

#[unsafe_no_drop_flag]
pub struct Rc<T> {
	ptr: *T
}

	impl<T:RefCounted> Rc<T> {
		
		pub fn new( mut thing: Box<T> ) -> Rc<T> {
			thing.get_rc_header().rc += 1;
			 Rc { ptr: unsafe { mem::transmute::<_,*T>( thing ) } }
		}
		
		pub fn from_raw( thing: Raw<T> ) -> Rc<T> {
			thing.get().get_rc_header().rc += 1;
			Rc { ptr: thing.ptr }
		}
		
		#[inline(always)]
		pub fn get( &self ) -> &mut T {
			unsafe { mem::transmute( self.ptr ) }
		}
		
		#[inline(always)]
		pub fn as_raw( &self ) -> Raw<T> {
			Raw { ptr: self.ptr }
		}
	}
	
	impl<T:RefCounted> Clone for Rc<T> {
		fn clone( &self ) -> Rc<T> {
			self.get().get_rc_header().rc += 1;
			Rc { ptr: self.ptr }
		}
	}
	
	#[unsafe_destructor]
	impl<T:RefCounted> Drop for Rc<T> {
		fn drop( &mut self ) {
			if ! self.ptr.is_null() {
				
				self.get().get_rc_header().rc -= 1;
				if self.get().get_rc_header().rc == 0 {
					drop( unsafe { mem::transmute::<_,Box<T>>( self.ptr ) } );
				}
				
				self.ptr = ptr::null();
			}
		}
	}

pub struct RcHeader {
	rc: uint,
}

	impl RcHeader {
		pub fn new() -> RcHeader {
			RcHeader {
				rc: 0,
			}
		}
	}

pub trait RefCounted {
	
	fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader;
}

#[cfg(test)]
mod test {
	
	use mem::rc::{Rc, RcHeader, RefCounted};
	
	struct Thing {
		rc: RcHeader,
		dropped: *mut bool,
	}
	
		impl Thing {
			
			pub fn new( dropped: *mut bool ) -> Thing {
				Thing {
					rc: RcHeader::new(),
					dropped: dropped,
				}
			}
		}
		
		impl RefCounted for Thing {
			
			fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
				&mut self.rc
			}
		}
		
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
		
		let r = Rc::new( box thing );
		assert!( ! dropped );
		
		drop( r );
		assert!( dropped );
	}
	
	#[test]
	fn test_clone() {
		let mut dropped = false;
		let thing = Thing::new( &mut dropped );
		
		let r1 = Rc::new( box thing );
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
	use mem::rc::{Rc, RcHeader, RefCounted};
	
	struct Thing {
		rc: RcHeader,
		a: int,
	}
	
		impl RefCounted for Thing {
			
			fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
				&mut self.rc
			}
		}
	
	#[bench]
	fn bench_raw_deref( bench: &mut Bencher ) {
		let thing = box Thing {
			rc: RcHeader::new(),
			a: 3,
		};
		let r: *Thing = unsafe { mem::transmute( thing ) };
		
		bench.iter( || {
			assert!( unsafe { (*r).a } == 3 );
		} );
	}
	
	#[bench]
	fn bench_rc_get( bench: &mut Bencher ) {
		let thing = box Thing {
			rc: RcHeader::new(),
			a: 3,
		};
		let r = Rc::new( thing );
		
		bench.iter( || {
			assert!( r.get().a == 3 );
		} );
	}
}
