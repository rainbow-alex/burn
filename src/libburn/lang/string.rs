use std::fmt;
use mem::rc::RefCounted;

pub struct String {
	value: StrBuf,
}

	impl String {
		
		pub fn new( value: StrBuf ) -> String {
			String {
				value: value,
			}
		}
		
		pub fn get_value<'l>( &'l self ) -> &'l str {
			self.value.as_slice()
		}
		
		pub fn len( &self ) -> uint {
			self.value.len()
		}
	}
	
	impl RefCounted for String {}
	
	impl fmt::Show for String {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			write!( f, "{}", self.value )
		}
	}
