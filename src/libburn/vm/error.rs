pub trait Error {
	fn get_message<'l>( &'l self ) -> &'l str;
	fn get_source_offset( &self ) -> uint;
}

pub struct ParseError {
	pub message: String,
	pub source_offset: uint,
}

	impl Error for ParseError {
		fn get_message<'l>( &'l self ) -> &'l str { self.message.as_slice() }
		fn get_source_offset( &self ) -> uint { self.source_offset }
	}

pub struct AnalysisError {
	pub message: String,
	pub source_offset: uint,
}

	impl Error for AnalysisError {
		fn get_message<'l>( &'l self ) -> &'l str { self.message.as_slice() }
		fn get_source_offset( &self ) -> uint { self.source_offset }
	}
