use collections::HashMap;
use lang::value;
use lang::identifier::Identifier;

pub struct Module {
	contents: HashMap<Identifier, value::Value>,
}

	impl Module {
		pub fn new() -> Module {
			Module {
				contents: HashMap::new(),
			}
		}
		
		pub fn get<'l>( &'l self, identifier: &'l Identifier ) -> &'l value::Value {
			self.contents.get( identifier )
		}
		
		pub fn set( &mut self, identifier: Identifier, value: value::Value ) {
			self.contents.insert( identifier, value );
		}
	}
