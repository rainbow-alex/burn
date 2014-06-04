use mem::raw::Raw;
use parse::node;
use lang::identifier::Identifier;

pub type Time = uint;

pub struct Frame {
	pub declared_variables: Vec<Box<Variable>>,
	pub functions: Vec<Raw<node::Expression>>,
	pub n_local_variables: uint,
	pub n_shared_local_variables: uint,
	pub closure: Option<Closure>,
}

	impl Frame {
		
		pub fn new() -> Frame {
			Frame {
				declared_variables: Vec::new(),
				functions: Vec::new(),
				n_local_variables: 0,
				n_shared_local_variables: 0,
				closure: None,
			}
		}
		
		pub fn new_with_closure() -> Frame {
			Frame {
				declared_variables: Vec::new(),
				functions: Vec::new(),
				n_local_variables: 0,
				n_shared_local_variables: 0,
				closure: Some( Closure::new() ),
			}
		}
		
		pub fn get_closure<'l>( &'l mut self ) -> &'l mut Closure {
			self.closure.as_mut().unwrap()
		}
	}
	
	impl PartialEq for Frame {
		fn eq( &self, other: &Frame ) -> bool {
			self as *Frame == other as *Frame
		}
	}
	
	impl Eq for Frame {}

pub struct Variable {
	pub name: Identifier,
	pub declared_in: Raw<Frame>,
	pub reads: Vec<ReadVariable>,
	pub writes: Vec<WriteVariable>,
	pub root_binds: Vec<BindVariable>,
	pub n_binds: uint,
	pub local_storage_type: storage::LocalStorageType,
	pub local_storage_index: uint,
	pub bound_storage_type: storage::BoundStorageType,
}

	impl Variable {
		
		pub fn new( name: Identifier ) -> Variable {
			Variable {
				name: name,
				declared_in: Raw::null(),
				reads: Vec::new(),
				writes: Vec::new(),
				root_binds: Vec::new(),
				n_binds: 0,
				local_storage_type: storage::SharedLocal,
				local_storage_index: 0,
				bound_storage_type: storage::SharedBound,
				// bound_storage_index differs per Binding
			}
		}
	}
	
	impl PartialEq for Variable {
		fn eq( &self, other: &Variable ) -> bool {
			self as *Variable == other as *Variable
		}
	}
	
	impl Eq for Variable {}

pub struct ReadVariable {
	pub time: Time,
}

pub struct WriteVariable {
	pub time: Time,
}

pub struct BindVariable {
	pub time: Time,
	pub mutable: bool,
}

pub struct Closure {
	pub created_at: Time,
	pub bindings: Vec<Binding>,
	pub n_static_bound_variables: uint,
	pub n_shared_bound_variables: uint,
}

	impl Closure {
		
		pub fn new() -> Closure {
			Closure {
				created_at: 0,
				bindings: Vec::new(),
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			}
		}
	}

pub struct Binding {
	pub variable: Raw<Variable>,
	pub mutable: bool,
	pub storage_index: uint,
}

pub struct Use {
	pub name: Identifier,
	pub operation: Raw<::lang::module::Use>,
}

	impl Use {
		
		pub fn new( name: Identifier ) -> Use {
			Use {
				name: name,
				operation: Raw::null(),
			}
		}
	}

pub struct Name {
	pub resolution: NameResolution,
}

	impl Name {
		
		pub fn new() -> Name {
			Name {
				resolution: Implicit,
			}
		}
	}

pub enum NameResolution {
	Implicit,
	Use( Raw<Use> ),
}

pub mod storage {
	
	pub enum LocalStorageType {
		Local,
		SharedLocal,
	}
	
	pub enum BoundStorageType {
		StaticBound,
		SharedBound,
	}
}
