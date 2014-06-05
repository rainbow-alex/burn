use lang::value::Value;

#[deriving(Clone)]
pub enum Flow {
	Running,
	Catching( Value ),
	Throwing( Value ),
	Returning( Value ),
	Jumping { pub n_flow_points: uint, pub instruction: uint },
}

	impl Flow {
		
		pub fn unwrap_throwable( self ) -> Value {
			match self {
				Catching( v ) | Throwing( v ) => v,
				_ => { unreachable!(); }
			}
		}
	}

pub enum FlowPoint {
	StartCatch { pub instruction: uint },
	StartFinally { pub instruction: uint },
	PopFrame { pub data_stack_len: uint },
	PopFrameAndRestoreFlow { pub data_stack_len: uint },
	PopSuppressedFlow,
}
