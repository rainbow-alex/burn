use lang::origin::Origin;
use lang::value::Value;
use mem::rc::Rc;
use vm::virtual_machine::VirtualMachine;

pub trait Error {
	fn get_message<'l>( &'l self ) -> &'l str;
	fn get_origin<'l>( &'l self ) -> &'l Origin;
	fn get_source_offset( &self ) -> uint;
}

pub struct ParseError {
	pub message: String,
	pub origin: Rc<Box<Origin>>,
	pub source_offset: uint,
}

	impl Error for ParseError {
		fn get_message<'l>( &'l self ) -> &'l str { self.message.as_slice() }
		fn get_origin<'l>( &'l self ) -> &'l Origin { let tmp: &Origin = *self.origin; tmp }
		fn get_source_offset( &self ) -> uint { self.source_offset }
	}

pub struct AnalysisError {
	pub message: String,
	pub origin: Rc<Box<Origin>>,
	pub source_offset: uint,
}

	impl Error for AnalysisError {
		fn get_message<'l>( &'l self ) -> &'l str { self.message.as_slice() }
		fn get_origin<'l>( &'l self ) -> &'l Origin { let tmp: &Origin = *self.origin; tmp }
		fn get_source_offset( &self ) -> uint { self.source_offset }
	}

pub trait UncaughtThrowableHandler {
	fn handle_uncaught_throwable( &mut self, &mut VirtualMachine, Value );
}
