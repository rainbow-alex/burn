use lang::value;
use mem::rc::RefCounted;

pub struct TypeUnion {
	pub left: value::Value,
	pub right: value::Value,
}

	impl TypeUnion {
		
		pub fn new( left: value::Value, right: value::Value ) -> TypeUnion {
			TypeUnion {
				left: left,
				right: right,
			}
		}
	}
	
	impl RefCounted for TypeUnion {}

pub struct TypeIntersection {
	pub left: value::Value,
	pub right: value::Value,
}

	impl RefCounted for TypeIntersection {}
