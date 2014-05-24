use lang::value;
use vm::frame::Frame;
use vm::flow;
use vm::flow::{Flow, FlowPoint};

pub struct Fiber {
	pub frames: Vec<Frame>,
	pub flow: Flow,
	pub flow_points: Vec<FlowPoint>,
	pub suppressed_flows: Vec<Flow>,
	pub data_stack: Vec<value::Value>,
}

	impl Fiber {
		
		pub fn new( frame: Frame ) -> Fiber {
			Fiber {
				frames: vec!( frame ),
				flow: flow::Running,
				flow_points: Vec::new(),
				suppressed_flows: Vec::new(),
				data_stack: Vec::new(),
			}
		}
	}
