use std::mem;
use lang::value;
use vm::run::frame::Frame;
use vm::run::flow;

pub struct Fiber {
	pub frame_stack: Vec<Frame>,
	pub frame: Frame,
	pub flow_points: Vec<flow::FlowPoint>,
	pub suppressed_flows: Vec<flow::Flow>,
	pub flow: flow::Flow,
	pub data_stack: Vec<value::Value>,
}

	impl Fiber {
		
		pub fn new( frame: Frame ) -> Fiber {
			Fiber {
				frame_stack: Vec::new(),
				frame: frame,
				flow_points: Vec::new(),
				suppressed_flows: Vec::new(),
				flow: flow::Running,
				data_stack: Vec::new(),
			}
		}
		
		pub fn pop_frame( &mut self ) -> Frame {
			mem::replace( &mut self.frame, self.frame_stack.pop().unwrap() )
		}
		
		pub fn push_frame( &mut self, frame: Frame ) {
			self.frame_stack.push( mem::replace( &mut self.frame, frame ) );
		}
		
		pub fn pop_data( &mut self ) -> value::Value {
			self.data_stack.pop().unwrap()
		}
		
		pub fn push_data( &mut self, value: value::Value ) {
			self.data_stack.push( value );
		}
	}
