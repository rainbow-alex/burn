use lang::value;
use mem::rc::{RcHeader, RefCounted};

pub struct TypeUnion {
	rc: RcHeader,
	pub left: value::Value,
	pub right: value::Value,
}

	impl TypeUnion {
		
		pub fn new( left: value::Value, right: value::Value ) -> TypeUnion {
			TypeUnion {
				rc: RcHeader::new(),
				left: left,
				right: right,
			}
		}
	}
	
	impl RefCounted for TypeUnion {
		fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
			&mut self.rc
		}
	}

pub struct TypeIntersection {
	rc: RcHeader,
	pub left: value::Value,
	pub right: value::Value,
}

	impl RefCounted for TypeIntersection {
		fn get_rc_header<'l>( &'l mut self ) -> &'l mut RcHeader {
			&mut self.rc
		}
	}
