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
	pub on_return: Option<proc(value::Value)>,
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
				on_return: None,
			}
		}
		
		pub fn pop_frame( &mut self ) -> Frame {
			mem::replace( &mut self.frame, self.frame_stack.pop().unwrap() )
		}
		
		pub fn push_frame( &mut self, frame: Frame ) {
			self.frame_stack.push( mem::replace( &mut self.frame, frame ) );
		}
		
		pub fn set_flow( &mut self, flow: flow::Flow ) {
			self.flow = flow;
		}
		
		pub fn replace_flow( &mut self, flow: flow::Flow ) -> flow::Flow {
			mem::replace( &mut self.flow, flow )
		}
		
		pub fn restore_flow( &mut self ) {
			self.flow = self.suppressed_flows.pop().unwrap();
		}
		
		pub fn pop_data( &mut self ) -> value::Value {
			self.data_stack.pop().unwrap()
		}
		
		pub fn push_data( &mut self, value: value::Value ) {
			self.data_stack.push( value );
		}
		
		pub fn end_return( self, value: value::Value ) {
			if self.on_return.is_some() {
				self.on_return.unwrap()( value );
			}
		}
	}
