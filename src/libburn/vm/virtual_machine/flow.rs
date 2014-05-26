use lang::value;

pub enum Flow {
	Running,
	Catching( value::Value ),
	Throwing( value::Value ),
	Returning( value::Value ),
	Jumping { pub n_flow_points: uint, pub instruction: uint },
}

pub enum FlowPoint {
	StartCatch { pub instruction: uint },
	StartFinally { pub instruction: uint },
	PopFrame,
	PopSuppressedFlow,
}
