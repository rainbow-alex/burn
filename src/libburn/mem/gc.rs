use std::mem;
use std::ptr;

#[unsafe_no_drop_flag]
pub struct Gc<T> {
	ptr: *mut GcWrapper<T>,
}

	impl<T:GarbageCollected> Gc<T> {
		
		#[inline(always)]
		pub fn get( &self ) -> &mut T {
			unsafe { &mut (*self.ptr).value }
			//&mut self.get_wrapper().value
		}
	}
	
	impl<T:GarbageCollected> Clone for Gc<T> {
		fn clone( &self ) -> Gc<T> {
			unsafe { (*self.ptr).rc += 1; }
			Gc { ptr: self.ptr }
		}
	}
	
	#[unsafe_destructor]
	impl<T:GarbageCollected> Drop for Gc<T> {
		fn drop( &mut self ) {
			unsafe {
				if ! self.ptr.is_null() {
					
					(*self.ptr).rc -= 1;
					if (*self.ptr).rc == 0 {
						self.get().die();
					}
					
					self.ptr = ptr::mut_null();
				}
			}
		}
	}

pub struct GcWrapper<T> {
	rc: uint,
	marked: bool,
	is_immortal: bool,
	value: T,
}

pub trait GarbageCollected {
	
	fn mark( &mut self );
	
	fn die( &mut self ) {
	}
}

pub struct GarbageCollectedManager<T> {
	alive: Vec<*mut GcWrapper<T>>,
	immortal: Vec<Box<GcWrapper<T>>>,
}

	impl<T:GarbageCollected> GarbageCollectedManager<T> {
		pub fn new() -> GarbageCollectedManager<T> {
			GarbageCollectedManager {
				alive: Vec::new(),
				immortal: Vec::new(),
			}
		}
		
		pub fn register( &mut self, thing: T ) -> Gc<T> {
			
			let mut gc_wrapper = box GcWrapper {
				rc: 1,
				marked: false,
				is_immortal: false,
				value: thing,
			};
			
			let ptr = &mut *gc_wrapper as *mut GcWrapper<T>;
			self.alive.push( ptr );
			unsafe { mem::forget( gc_wrapper ); }
			
			Gc { ptr: ptr }
		}
		
		pub fn sweep( &mut self ) {
			unsafe {
				let mut i = 0;
				let mut end = 0;
				
				let n = self.alive.len();
				while i < n {
					
					let ptr = *self.alive.get( i );
					
					if (*ptr).is_immortal {
						
						let owned = mem::transmute::<_,Box<GcWrapper<T>>>( ptr );
						self.immortal.push( owned );
						i += 1;
						
					} else if (*ptr).marked {
						
						(*ptr).marked = false;
						i += 1;
						end += 1;
						
					} else {
						
						let mut owned = mem::transmute::<_,Box<GcWrapper<T>>>( ptr );
						
						// if the rc is not 0, this was part of some cycle,
						// and die() was not yet called
						if owned.rc != 0 {
							owned.value.die();
						}
						
						drop( owned );
						i += 1;
					}
				}
				
				self.alive.truncate( end );
			}
		}
	}
	
	#[unsafe_destructor]
	impl<T:GarbageCollected> Drop for GarbageCollectedManager<T> {
		fn drop( &mut self ) {
			unsafe {
				for &t in self.alive.iter() {
					let mut owned = mem::transmute::<_,Box<GcWrapper<T>>>( t );
					
					if owned.rc != 0 {
						owned.value.die();
					}
					
					drop( owned );
				}
			}
		}
	}

#[cfg(test)]
mod test {
	
	use super::{GarbageCollected, GarbageCollectedManager};
	
	struct Thing {
		dropped: *mut bool,
	}
	
		impl GarbageCollected for Thing {
			
			fn mark( &mut self ) {
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
	fn test() {
		
		let mut things = GarbageCollectedManager::<Thing>::new();
		let mut dropped = false;
		
		assert!( things.alive.len() == 0 );
		
		let thing = things.register( Thing { dropped: &mut dropped } );
		assert!( things.alive.len() == 1 );
		
		let thing2 = thing.clone();
		assert!( things.alive.len() == 1 );
		
		drop( thing );
		assert!( things.alive.len() == 1 );
		
		drop( thing2 );
		assert!( things.alive.len() == 1 );
		assert!( dropped == false );
		
		things.sweep();
		assert!( things.alive.len() == 0 );
		assert!( dropped == true );
	}
}
