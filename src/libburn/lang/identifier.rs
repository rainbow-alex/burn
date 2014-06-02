use std::hash::Hash;
use std::hash::sip::SipState;
use std::mem;
use std::fmt;
use collections::HashMap;
use mem::raw::Raw;

static mut ALL: Raw<HashMap<String,Box<IdentifierContainer>>> = Raw { ptr: 0 as *mut _ };

struct IdentifierContainer {
	value: String,
}

#[deriving(Copy, PartialEq, Eq)]
pub struct Identifier {
	ptr: Raw<IdentifierContainer>,
}

	impl Identifier {
		
		pub fn find_or_create( value: &str ) -> Identifier {
			unsafe {
				
				if ALL.is_null() {
					let all = box HashMap::<String,Box<IdentifierContainer>>::new();
					ALL = Raw::new( all );
					mem::forget( all );
				}
				
				let container = ALL.get().find_or_insert_with(
					value.into_string(),
					|_| { box IdentifierContainer { value: value.into_string() } }
				);
				
				Identifier { ptr: Raw::new( *container ) }
			}
		}
		
		pub fn get_value<'l>( &'l self ) -> &'l str {
			self.ptr.get().value.as_slice()
		}
	}
	
	impl Hash for Identifier {
		fn hash( &self, state: &mut SipState ) {
			self.ptr.ptr.hash( state );
		}
	}
	
	impl Clone for Identifier {
		fn clone( &self ) -> Identifier {
			*self
		}
	}
	
	impl fmt::Show for Identifier {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			write!( f, "{}", self.ptr.get().value )
		}
	}

#[cfg(test)]
mod test {
	
	use lang::identifier::Identifier;
	
	#[test]
	fn test() {
		let id = Identifier::find_or_create( "test" );
		let id2 = Identifier::find_or_create( "test" );
		assert!( id.ptr == id2.ptr );
	}
}
