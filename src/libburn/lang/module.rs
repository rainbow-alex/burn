use collections::HashMap;
use lang::value;
use lang::identifier::Identifier;
use mem::raw::Raw;

pub struct Module {
	modules: HashMap<Identifier, Box<Module>>,
	contents: HashMap<Identifier, value::Value>,
	locked: bool,
}

	impl Module {
		pub fn new() -> Module {
			Module {
				modules: HashMap::new(),
				contents: HashMap::new(),
				locked: false,
			}
		}
		
		pub fn add_module( &mut self,  name: &'static str, module: Box<Module> ) {
			assert!( ! self.locked );
			let name = Identifier::find_or_create( name );
			self.contents.insert( name, value::Module( Raw::new( module ) ) );
			self.modules.insert( name, module );
		}
		
		pub fn add( &mut self, name: &'static str, value: value::Value ) {
			assert!( ! self.locked );
			self.contents.insert( Identifier::find_or_create( name ), value );
		}
		
		pub fn lock( &mut self ) {
			assert!( ! self.locked );
			self.locked = true
		}
		
		#[inline]
		pub fn get<'l>( &'l self, identifier: Identifier ) -> &'l value::Value {
			self.contents.get( &identifier )
		}
	}
