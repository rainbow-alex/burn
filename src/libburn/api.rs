pub mod lang {
	pub use lang::identifier::Identifier;
	pub use lang::function::Function;
	pub use lang::module::Module;
	pub use lang::special::{Special, RefCountedSpecial, StaticSpecialDef, StaticSpecial};
	
	pub mod value {
		pub use lang::value::Value;
	}
}

pub mod mem {
	pub use mem::rc::{Rc, RefCounted};
	pub use mem::gc::{Gc, GarbageCollected};
}

pub mod vm {
	pub use vm::error::Error;
	pub use vm::virtual_machine::VirtualMachine;
	pub use vm::result;
}

pub mod repl {
	pub use vm::repl::State;
}
