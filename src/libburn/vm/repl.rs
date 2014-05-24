use lang::value;
use lang::identifier::Identifier;
use collections::HashMap;

pub struct ReplState {
	pub variables: HashMap<Identifier, value::SharedValue>,
}

	impl ReplState {
		
		pub fn new() -> ReplState {
			ReplState {
				variables: HashMap::new(),
			}
		}
		
		pub fn declare_variable( &mut self, name: Identifier ) {
			self.variables.insert( name, value::SharedValue::new( value::Nothing ) );
		}
	}
