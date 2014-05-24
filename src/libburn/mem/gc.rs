use std::mem;
use mem::raw::Raw;

#[unsafe_no_drop_flag]
pub struct Gc<T> {
	ptr: *T,
}

	impl<T:GarbageCollected> Gc<T> {
		
		pub fn get( &self ) -> &mut T {
			unsafe { mem::transmute( self.ptr ) }
		}
		
		pub fn mark( &self ) {
			let thing = self.get();
			thing.get_gc_header().marked = true;
			thing.mark();
		}
		
		pub fn as_raw( &self ) -> Raw<T> {
			Raw { ptr: self.ptr }
		}
	}
	
	impl<T:GarbageCollected> Clone for Gc<T> {
		fn clone( &self ) -> Gc<T> {
			self.get().get_gc_header().rc += 1;
			Gc { ptr: self.ptr }
		}
	}
	
	#[unsafe_destructor]
	impl<T:GarbageCollected> Drop for Gc<T> {
		fn drop( &mut self ) {
			if self.ptr != 0 as *T {
				
				let thing = self.get();
				thing.get_gc_header().rc -= 1;
				if thing.get_gc_header().rc == 0 {
					thing.die();
				}
				
				self.ptr = 0 as *T;
			}
		}
	}

pub struct GcHeader {
	rc: uint,
	marked: bool,
	is_immortal: bool,
}

	impl GcHeader {
		pub fn new() -> GcHeader {
			GcHeader {
				rc: 0,
				marked: false,
				is_immortal: false,
			}
		}
	}

pub trait GarbageCollected {
	fn get_gc_header<'l>( &'l mut self ) -> &'l mut GcHeader;
	fn mark( &mut self ) { fail!( "TODO" ); }
	fn become_immortal( &mut self ) { fail!( "TODO" ); }
	fn die( &mut self ) {}
}

pub struct GarbageCollectedManager<T> {
	alive: Vec<Raw<T>>,
	immortal: Vec<Box<T>>,
}

	impl<T:GarbageCollected> GarbageCollectedManager<T> {
		pub fn new() -> GarbageCollectedManager<T> {
			GarbageCollectedManager {
				alive: Vec::new(),
				immortal: Vec::new(),
			}
		}
		
		pub fn register( & mut self, mut thing: Box<T> ) -> Gc<T> {
			thing.get_gc_header().rc += 1;
			
			let gc = Gc { ptr: &*thing as *T };
			self.alive.push( gc.as_raw() );
			unsafe { mem::forget( thing ); }
			
			gc
		}
		
		pub fn register_immortal( &mut self, mut thing: Box<T> ) -> Gc<T> {
			thing.get_gc_header().is_immortal = true;
			thing.become_immortal();
			
			let gc = Gc { ptr: &*thing as *T };
			self.immortal.push( thing );
			
			gc
		}
		
		pub fn sweep( &mut self ) {
			unsafe {
				let mut i = 0;
				let mut end = 0;
				
				let n = self.alive.len();
				while i < n {
					
					let raw = self.alive.get( i );
					let gc_header = raw.get().get_gc_header();
					
					if gc_header.is_immortal {
						
						let owned = mem::transmute::<_,Box<T>>( raw.ptr );
						self.immortal.push( owned );
						i += 1;
						
					} else if gc_header.marked {
						
						gc_header.marked = false;
						i += 1;
						end += 1;
						
					} else {
						
						let mut owned = mem::transmute::<_,Box<T>>( raw.ptr );
						
						// if the rc is not 0, this was part of some cycle, and die() was not yet called
						if owned.get_gc_header().rc != 0 {
							owned.die();
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
			for t in self.alive.iter() {
				unsafe {
					let mut owned = mem::transmute::<_,Box<T>>( *t );
					
					if owned.get_gc_header().rc != 0 {
						owned.die();
					}
					
					drop( owned );
				}
			}
		}
	}

#[cfg(test)]
mod test {
	
	use mem::gc::{Gc, GcHeader, GarbageCollected, GarbageCollectedManager};
	
	struct Thing {
		gc: GcHeader,
		refs: Vec<Gc<Thing>>,
		freed: bool,
		dropped: *mut bool,
	}
	
		impl Thing {
			fn new( dropped: *mut bool ) -> Thing {
				Thing {
					gc: GcHeader::new(),
					refs: Vec::new(),
					freed: false,
					dropped: dropped,
				}
			}
		}
		
		impl GarbageCollected for Thing {
			
			fn get_gc_header<'l>( &'l mut self ) -> &'l mut GcHeader {
				&mut self.gc
			}
			
			fn mark( &mut self ) {
				for r in self.refs.iter() {
					r.mark();
				}
			}
			
			fn become_immortal( &mut self ) {
				// nop
			}
			
			fn die( &mut self ) {
				assert!( ! self.freed );
				self.freed = true;
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
	fn test_refcounting() {
		
		let mut manager = GarbageCollectedManager::<Thing>::new();
		
		assert!( manager.alive.len() == 0 );
		
		let mut dropped = false;
		let thing = manager.register( box Thing::new( &mut dropped as *mut bool ) );
		assert!( manager.alive.len() == 1 );
		assert!( thing.get().get_gc_header().rc == 1 );
		
		let thing2 = thing.clone();
		assert!( manager.alive.len() == 1 );
		assert!( thing.get().get_gc_header().rc == 2 );
		assert!( thing2.get().get_gc_header().rc == 2 );
		
		drop( thing );
		assert!( manager.alive.len() == 1 );
		assert!( thing2.get().get_gc_header().rc == 1 );
		
		drop( thing2 );
		assert!( manager.alive.len() == 1 );
		assert!( dropped == false );
		
		manager.sweep();
		assert!( manager.alive.len() == 0 );
		assert!( dropped == true );
	}
}
