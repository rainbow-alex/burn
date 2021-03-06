use lang::function::Function;
use mem::gc::Gc;
use mem::rc::{Rc, RefCounted};
use mem::raw::Raw;
use lang::type_::{TypeUnion, TypeIntersection};
use lang::module::Module;
use lang::special::{StaticSpecial, RcSpecial};

#[deriving(Clone)]
pub enum Value {
	
	#[doc(hidden)]
	Nothing,
	#[doc(hidden)]
	Boolean( bool ),
	#[doc(hidden)]
	Integer( i64 ),
	#[doc(hidden)]
	Float( f64 ),
	#[doc(hidden)]
	String( Rc<String> ),
	
	#[doc(hidden)]
	Function( Gc<Function> ),
	#[doc(hidden)]
	TypeUnion( Rc<TypeUnion> ),
	#[doc(hidden)]
	TypeIntersection( Rc<TypeIntersection> ),
	#[doc(hidden)]
	Module( Raw<Module> ),
	
	#[doc(hidden)]
	StaticSpecial( StaticSpecial ),
	#[doc(hidden)]
	RcSpecial( Rc<RcSpecial> ),
}

	impl RefCounted for Value {}
