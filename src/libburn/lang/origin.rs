use std::fmt;
use mem::rc::RefCounted;

pub trait Origin {
	fn get_name<'l>( &'l self ) -> &'l str;
	fn get_path<'l>( &'l self ) -> Option<&'l Path>;
}

	impl RefCounted for Box<Origin> {}
	
	impl fmt::Show for Box<Origin> {
		fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
			write!( f, "{}", self.get_name() )
		}
	}

pub struct Script {
	pub path: Path,
}

	impl Origin for Script {
		
		fn get_name<'l>( &'l self ) -> &'l str {
			self.path.as_str().unwrap()
		}
		
		fn get_path<'l>( &'l self ) -> Option<&'l Path> {
			Some( &self.path )
		}
	}

pub struct Stdin;

	impl Origin for Stdin {
		
		fn get_name<'l>( &'l self ) -> &'l str {
			"<stdin>"
		}
		
		fn get_path<'l>( &'l self ) -> Option<&'l Path> {
			None
		}
	}

pub struct Rust {
	pub name: String,
}

	impl Origin for Rust {
		
		fn get_name<'l>( &'l self ) -> &'l str {
			self.name.as_slice()
		}
		
		fn get_path<'l>( &'l self ) -> Option<&'l Path> {
			None
		}
	}
