use mem::rc::Rc;
use lang::value;
use lang::identifier::Identifier;
use collections::HashMap;

pub struct State {
	pub variables: HashMap<Identifier, Rc<value::Value>>,
}

	impl State {
		
		pub fn new() -> State {
			State {
				variables: HashMap::new(),
			}
		}
		
		pub fn declare_variable( &mut self, name: Identifier ) {
			self.variables.insert( name, Rc::new( value::Nothing ) );
		}
	}
