use parse::node;
use mem::raw::Raw;
use lang::identifier::Identifier;

#[path="analysis/1_find_variable_declarations.rs"]
pub mod find_variable_declarations;
#[path="analysis/2_resolve_variables_and_set_times.rs"]
pub mod resolve_variables_and_set_times;
#[path="analysis/3_determine_variable_lifetime_and_storage_type.rs"]
pub mod determine_variable_lifetime_and_storage_type;
#[path="analysis/4_determine_allocation.rs"]
pub mod determine_allocation;

type Time = uint;

pub struct FrameAnalysis {
	pub n_local_variables: uint,
	pub n_shared_local_variables: uint,
}

	impl FrameAnalysis {
		
		pub fn new() -> FrameAnalysis {
			FrameAnalysis {
				n_local_variables: 0,
				n_shared_local_variables: 0,
			}
		}
	}
	
	impl Eq for FrameAnalysis {
		fn eq( &self, other: &FrameAnalysis ) -> bool {
			self as *FrameAnalysis == other as *FrameAnalysis
		}
	}

pub struct ClosureAnalysis {
	pub frame: FrameAnalysis,
	created_at: Time,
	pub bound: Vec<Binding>,
	pub n_static_bound_variables: uint,
	pub n_shared_bound_variables: uint,
}

	impl ClosureAnalysis {
		
		pub fn new() -> ClosureAnalysis {
			ClosureAnalysis {
				frame: FrameAnalysis::new(),
				created_at: 0,
				bound: Vec::new(),
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

pub struct ScopeAnalysis {
	pub frame: Raw<FrameAnalysis>,
	start_at: Time,
	end_at: Time,
	is_loop: bool,
	pub declared: Vec<Raw<VariableAnalysis>>,
	functions: Vec<Raw<node::Expression>>,
}

	impl ScopeAnalysis {
		
		pub fn new() -> ScopeAnalysis {
			ScopeAnalysis {
				frame: Raw::null(),
				start_at: 0,
				end_at: 0,
				is_loop: false,
				declared: Vec::new(),
				functions: Vec::new(),
			}
		}
	}

pub struct VariableAnalysis {
	pub name: Identifier,
	pub declared_in: Raw<ScopeAnalysis>,
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
