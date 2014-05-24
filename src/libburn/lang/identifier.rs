use std::hash::Hash;
use std::hash::sip::SipState;
use std::mem;
use std::fmt;
use collections::HashMap;
use mem::raw::Raw;

static mut ALL: Raw<HashMap<StrBuf,Box<IdentifierContainer>>> = Raw { ptr: 0 as *_ };

#[inline(always)]
pub fn id( value: &str ) -> Identifier {
	Identifier::find_or_create( value )
}

struct IdentifierContainer {
	value: StrBuf,
}

#[deriving(Copy, Eq, TotalEq)]
pub struct Identifier {
	ptr: Raw<IdentifierContainer>,
}

	impl Identifier {
		
		pub fn find_or_create( value: &str ) -> Identifier {
			unsafe {
				
				if ALL.is_null() {
					let all = box HashMap::<StrBuf,Box<IdentifierContainer>>::new();
					ALL = Raw::new( all );
					mem::forget( all );
				}
				
				let container = ALL.get().find_or_insert_with(
					value.into_owned(),
					|_| { box IdentifierContainer { value: value.into_owned() } }
				);
				
				Identifier { ptr: Raw::new( *container ) }
			}
		}
	}
	
	impl Hash for Identifier {
		fn hash( &self, state: &mut SipState ) {
			self.ptr.ptr.hash( state );
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
