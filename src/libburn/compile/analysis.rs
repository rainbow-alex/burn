use parse::node;
use mem::raw::Raw;
use lang::identifier::Identifier;

#[path="analysis/1_analyze_variables.rs"]
pub mod analyze_variables;
#[path="analysis/2_determine_allocation.rs"]
pub mod determine_allocation;

type Time = uint;

pub struct FrameAnalysis {
	pub declared: Vec<Box<VariableAnalysis>>,
	pub functions: Vec<Raw<node::Expression>>,
	pub n_local_variables: uint,
	pub n_shared_local_variables: uint,
	pub closure: Option<ClosureAnalysis>,
}

	impl FrameAnalysis {
		
		pub fn new() -> FrameAnalysis {
			FrameAnalysis {
				declared: Vec::new(),
				functions: Vec::new(),
				n_local_variables: 0,
				n_shared_local_variables: 0,
				closure: None,
			}
		}
		
		pub fn new_with_closure() -> FrameAnalysis {
			FrameAnalysis {
				declared: Vec::new(),
				functions: Vec::new(),
				n_local_variables: 0,
				n_shared_local_variables: 0,
				closure: Some( ClosureAnalysis::new() ),
			}
		}
		
		pub fn get_closure<'l>( &'l mut self ) -> &'l mut ClosureAnalysis {
			self.closure.as_mut().unwrap()
		}
	}
	
	impl Eq for FrameAnalysis {
		fn eq( &self, other: &FrameAnalysis ) -> bool {
			self as *FrameAnalysis == other as *FrameAnalysis
		}
	}

pub struct ClosureAnalysis {
	created_at: Time,
	pub bindings: Vec<Binding>,
	pub n_static_bound_variables: uint,
	pub n_shared_bound_variables: uint,
}

	impl ClosureAnalysis {
		
		pub fn new() -> ClosureAnalysis {
			ClosureAnalysis {
				created_at: 0,
				bindings: Vec::new(),
				n_static_bound_variables: 0,
				n_shared_bound_variables: 0,
			}
		}
	}

pub struct Binding {
	pub variable: Raw<VariableAnalysis>,
	mutable: bool,
	pub storage_index: uint,
}

pub struct VariableAnalysis {
	pub name: Identifier,
	pub declared_in: Raw<FrameAnalysis>,
	reads: Vec<ReadVariable>,
	writes: Vec<WriteVariable>,
	root_binds: Vec<BindVariable>,
	n_binds: uint,
	pub local_storage_type: StorageType,
	pub local_storage_index: uint,
	pub bound_storage_type: BoundStorageType,
}

	impl VariableAnalysis {
		
		pub fn new( name: Identifier ) -> VariableAnalysis {
			VariableAnalysis {
				name: name,
				declared_in: Raw::null(),
				reads: Vec::new(),
				writes: Vec::new(),
				root_binds: Vec::new(),
				n_binds: 0,
				local_storage_type: SharedLocalStorage,
				local_storage_index: 0,
				bound_storage_type: SharedBoundStorage,
				// bound_storage_index differs per Binding
			}
		}
	}
	
	impl Eq for VariableAnalysis {
		fn eq( &self, other: &VariableAnalysis ) -> bool {
			self as *VariableAnalysis == other as *VariableAnalysis
		}
	}

pub struct ReadVariable {
	time: Time,
}

pub struct WriteVariable {
	time: Time,
}

pub struct BindVariable {
	time: Time,
	mutable: bool,
}

pub enum StorageType {
	LocalStorage,
	SharedLocalStorage,
}

pub enum BoundStorageType {
	StaticBoundStorage,
	SharedBoundStorage,
}
