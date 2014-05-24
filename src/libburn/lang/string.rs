use std::fmt;
use mem::rc::{RcHeader, RefCounted};

pub struct String {
	rc: RcHeader,
	value: StrBuf,
}

	impl String {
		
		pub fn new( value: StrBuf ) -> String {
			String {
				rc: RcHeader::new(),
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
	
	impl RefCounted for String {
		fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
			&mut self.rc
		}
	}
	
	impl fmt::Show for String {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			write!( f, "{}", self.value )
		}
	}
