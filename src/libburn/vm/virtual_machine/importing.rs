use lang::identifier::Identifier;
use lang::value;
use vm::virtual_machine::{VirtualMachine, VirtualMachineImporting};

pub type ImportId = uint;

pub enum ImportCacheEntry {
	NotLoaded( Vec<Identifier> ),
	Loaded( value::Value ),
}

	impl ImportCacheEntry {
		
		#[inline]
		fn is_loaded( &self ) -> bool {
			match *self {
				NotLoaded(..) => false,
				Loaded(..) => true,
			}
		}
	}

impl VirtualMachineImporting for VirtualMachine {
	
	fn find_import( &mut self, fqn: Vec<Identifier> ) -> (ImportId, bool) {
		match self.import_ids_by_fqn.find( &fqn ) {
			Some( &id ) => (id, self.import_cache.get( id ).is_loaded()),
			None => {
				let id = self.import_cache.len();
				self.import_cache.push( NotLoaded( fqn ) );
				(id, false)
			}
		}
	}
	
	fn import_or_get_cached( &mut self, id: ImportId ) -> Result<value::Value,value::Value> {
		
		let fqn = match *self.import_cache.get( id ) {
			Loaded( ref value ) => {
				return Ok( value.clone() );
			}
			NotLoaded( ref fqn ) => fqn.clone(),
		};
		
		match self.import( fqn ) {
			Ok( value ) => {
				*self.import_cache.get_mut( id ) = Loaded( value.clone() );
				Ok( value )
			},
			Err( e ) => Err( e )
		}
	}
}

impl ::vm::virtual_machine::VirtualMachine {
	
	fn import( &mut self, fqn: Vec<Identifier> ) -> Result<value::Value, value::Value> {
		
		let loaded = match self.root_import( *fqn.get(0) ) {
			Ok( x ) => x,
			Err( e ) => { return Err( e ); }
		};
		
		// TODO traverse
		
		Ok( loaded )
	}
	
	#[inline]
	fn root_import( &mut self, name: Identifier ) -> Result<value::Value, value::Value> {
		
		let mut i = 0;
		while i < self.import_paths.len() {
			
			let mut suspect = self.import_paths.get( i ).clone();
			suspect.push( name.get_value().to_owned().append( ".burn" ) );
			
			if suspect.exists() {
				let source = ::std::io::File::open( &suspect ).unwrap().read_to_str().unwrap();
				self.run_script( source.as_slice() );
				return Ok( value::Nothing )
			}
			
			i += 1;
		}
		
		return Err( value::Nothing );
	}
}
