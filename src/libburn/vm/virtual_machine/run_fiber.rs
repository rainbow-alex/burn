use std::mem;
use mem::gc::Gc;
use mem::raw::Raw;
use lang::identifier::Identifier;
use lang::function::Function;
use lang::value;
use vm::bytecode::opcode;
use vm::virtual_machine::flow;
use vm::virtual_machine::frame::Frame;
use vm::virtual_machine::fiber::Fiber;
use vm::virtual_machine::{VirtualMachine, VirtualMachineRunFiber, VirtualMachineImporting, result};
use builtin::burn::{operations, types, errors};

impl VirtualMachineRunFiber for VirtualMachine {
	
	fn run_fiber( &mut self, fiber: Fiber ) -> result::Result {
		
		let Fiber {
			flow: mut flow,
			frames: mut frames,
			flow_points: mut flow_points,
			suppressed_flows: mut suppressed_flows,
			data_stack: mut data_stack,
		} = fiber;
		
		let mut frame = frames.pop().unwrap();
		
		'frame_loop: loop {
			
			let code = Raw::new( frame.get_code() );
			let opcodes = code.get().opcodes.as_slice();
			
			'flow_loop: loop {
				
				match flow {
					
					flow::Running | flow::Catching(..) => {
						
						'instruction_loop: loop {
							
							debug!( {
								println!(
									"VM: running {}/{} ({})",
									frame.instruction,
									opcodes.len(),
									flow_points.len()
								);
							} )
							
							match opcodes[ frame.instruction ] {
								
								// Temporary ///////////////////////////////////////////////////
								
								opcode::Print => {
									println!( "{}", data_stack.pop().unwrap().to_string() );
								}
								
								// VM Commands /////////////////////////////////////////////////
								
								opcode::Nop => {
									// nop!
								}
								
								opcode::End => {
									return result::End;
								}
								
								opcode::ReturnPop => {
									return result::Value( data_stack.pop().unwrap() );
								}
								
								// Flow ////////////////////////////////////////////////////////
								
								opcode::PopFlowPoint => {
									flow_points.pop();
								}
								
								opcode::Jump { instruction: i } => {
									frame.instruction = i;
									continue 'instruction_loop;
								}
								
								opcode::JumpIfPopFalsy { instruction: i } => {
									if ! operations::is_truthy( & data_stack.pop().unwrap() ) {
										frame.instruction = i;
										continue 'instruction_loop;
									}
								}
								
								opcode::FlowJump { n_flow_points: n, instruction: i } => {
									flow = flow::Jumping { n_flow_points: n, instruction: i };
								}
								
								// Function flow ///////////////////////////////////////////////
								
								opcode::Call { n_arguments: n_arguments } => {
									
									let offset = data_stack.len() - ( 1 + n_arguments );
									let function_value = Raw::new( data_stack.get( offset ) );
									
									match *function_value.get() {
										
										value::Function( ref function ) => {
											
											let function: Gc<Function> = unsafe { mem::transmute_copy( function ) };
											
											assert!( n_arguments == 0 ); // TODO
											let locals = Vec::new();
											let shared = Vec::new();
											
											unsafe { data_stack.set_len( offset ); }
											
											frame.instruction += 1;
											
											frames.push( frame );
											flow_points.push( flow::PopFrame );
											
											frame = Frame::new_function( function, locals, shared );
											
											continue 'frame_loop;
										}
										
										value::StaticSpecial( ref r ) => {
											(r);
											fail!( "TODO" );
										}
										
										_ => {
											fail!( "TODO" );
										}
									}
								}
								
								opcode::Return => {
									flow = flow::Returning( data_stack.pop().unwrap() );
									continue 'flow_loop;
								}
								
								// Try/Catch/... ///////////////////////////////////////////////
								
								opcode::PushStartCatchFlowPoint { instruction: i } => {
									flow_points.push( flow::StartCatch { instruction: i } );
								}
								
								opcode::PushStartFinallyFlowPoint { instruction: i } => {
									flow_points.push( flow::StartFinally { instruction: i } );
								}
								
								opcode::Throw => {
									let throwable = data_stack.pop().unwrap();
									
									if types::is_throwable( &throwable ) {
										flow = flow::Throwing( throwable );
									} else {
										flow = flow::Throwing( errors::create_type_error( format!( "{} is not Throwable.", throwable.repr() ) ) );
									}
									
									continue 'flow_loop;
								}
								
								opcode::CatchOrJump { instruction: i } => {
									
									let throwable = match flow {
										flow::Catching( t ) => t,
										_ => fail!(),
									};
									
									let type_ = data_stack.pop().unwrap();
									let result = operations::is( &throwable, &type_ );
									
									match result {
										Ok( true ) => {
											*frame.local_variables.get_mut( 0 ) = throwable;
											flow = flow::Running;
										},
										Ok( false ) => {
											frame.instruction = i;
											flow = flow::Catching( throwable );
											continue 'instruction_loop;
										}
										Err( e ) => {
											flow = flow::Throwing( e );
											continue 'flow_loop;
										}
									}
								}
								
								opcode::Catch => {
									
									let throwable = match flow {
										flow::Catching( t ) => t,
										_ => fail!(),
									};
									
									*frame.local_variables.get_mut( 0 ) = throwable;
									flow = flow::Running;
								}
								
								opcode::Rethrow => {
									
									flow = match flow {
										flow::Catching( e ) => flow::Throwing( e ),
										_ => fail!(),
									};
									
									continue 'flow_loop;
								}
								
								opcode::StartFinally => {
									flow_points.pop();
									suppressed_flows.push( flow::Running );
									flow_points.push( flow::PopSuppressedFlow );
								}
								
								opcode::EndFinally => {
									flow_points.pop();
									flow = suppressed_flows.pop().unwrap();
									match flow {
										flow::Running => {},
										_ => continue 'flow_loop,
									};
								}
								
								// Data stack operations ///////////////////////////////////////
								
								opcode::PushFunction { index: i } => {
									
									data_stack.push(
										value::Function(
											self.functions.register(
												Function::new(
													code.get().functions.get( i ).clone()
												)
											)
										)
									);
								}
								
								opcode::PushString { index: i } => {
									data_stack.push( value::String( code.get().strings.get( i ).clone() ) );
								}
								
								opcode::PushFloat { value: f } => {
									data_stack.push( value::Float( f ) );
								}
								
								opcode::PushInteger { value: i } => {
									data_stack.push( value::Integer( i ) );
								}
								
								opcode::PushBoolean { value: b } => {
									data_stack.push( value::Boolean( b ) );
								}
								
								opcode::PushNothing => {
									data_stack.push( value::Nothing );
								}
								
								opcode::Pop => {
									data_stack.pop();
								}
								
								// Variables ///////////////////////////////////////////////////
								
								opcode::StoreLocal { index: i } => {
									*frame.local_variables.get_mut( i ) = data_stack.pop().unwrap();
								}
								
								opcode::LoadLocal { index: i } => {
									data_stack.push( frame.local_variables.get( i ).clone() );
								}
								
								opcode::StoreSharedLocal { index: i } => {
									*frame.shared_local_variables.get( i ).get() = data_stack.pop().unwrap();
								}
								
								opcode::LoadSharedLocal { index: i } => {
									data_stack.push( frame.shared_local_variables.get( i ).get().clone() );
								}
								
								opcode::StoreStaticBound { index: i } => {
									*frame.get_function().static_bound_variables.get_mut( i ) = data_stack.pop().unwrap();
								}
								
								opcode::LoadStaticBound { index: i } => {
									data_stack.push( frame.get_function().static_bound_variables.get( i ).clone() );
								}
								
								opcode::StoreSharedBound { index: i } => {
									*frame.get_function().shared_bound_variables.get( i ).get() = data_stack.pop().unwrap();
								}
								
								opcode::LoadSharedBound { index: i } => {
									data_stack.push( frame.get_function().shared_bound_variables.get( i ).get().clone() );
								}
								
								// Names ///////////////////////////////////////////////////////
								
								opcode::Import { id: _ } => {
									let (id,_) = self.find_import( vec!( Identifier::find_or_create( "foo" ) ) );
									let _ = self.import_or_get_cached( id );
								}
								
								opcode::LoadImplicit { name: name } => {
									data_stack.push( self.implicit.get().get( name ).clone() );
								}
								
								// Operators ///////////////////////////////////////////////////
								
								opcode::Is => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::is( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Eq => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::eq( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Neq => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::neq( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Lt => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::lt( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Gt => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::gt( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::LtEq => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::lt_eq( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::GtEq => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::gt_eq( &left, &right ) {
										Ok( result ) => {
											data_stack.push( value::Boolean( result ) );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Union => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::union( left, right ) {
										Ok( result ) => {
											data_stack.push( result );
										}
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Add => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::add( &left, &right ) {
										Ok( result ) => data_stack.push( result ),
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
								opcode::Subtract => {
									let right = data_stack.pop().unwrap();
									let left = data_stack.pop().unwrap();
									match operations::subtract( &left, &right ) {
										Ok( result ) => data_stack.push( result ),
										Err( err ) => {
											flow = flow::Throwing( err );
											continue 'flow_loop;
										}
									};
								}
								
							} // match opcodes[ frame.instruction ]
							
							frame.instruction += 1;
							
						} // 'instruction_loop
						
					} // flow::Running | flow::Catching(..)
					
					flow::Jumping { n_flow_points: mut n_flow_points, instruction: instruction } => {
						
						while n_flow_points > 0 {
							
							match flow_points.pop().unwrap() {
								
								flow::StartCatch {..} => {
									// ignored, there is no throwable that needs to be caught
								}
								
								flow::StartFinally { instruction: i } => {
									suppressed_flows.push( flow::Jumping { n_flow_points: n_flow_points, instruction: instruction } );
									flow = flow::Running;
									flow_points.push( flow::PopSuppressedFlow );
									frame.instruction = i;
									continue 'flow_loop;
								}
								
								flow::PopFrame => fail!(),
								
								flow::PopSuppressedFlow => {
									suppressed_flows.pop();
								}
							}
							
							n_flow_points -= 1;
						}
						
						frame.instruction = instruction;
						
					} // flow::Jumping( e )
					
					flow::Returning( value ) => {
						
						loop {
							match flow_points.pop().unwrap() {
								
								flow::StartCatch {..} => {
									// ignored, there is no throwable that needs to be caught
								}
								
								flow::StartFinally { instruction: i } => {
									suppressed_flows.push( flow::Returning( value ) );
									flow = flow::Running;
									flow_points.push( flow::PopSuppressedFlow );
									frame.instruction = i;
									continue 'flow_loop;
								}
								
								flow::PopFrame => {
									frame = frames.pop().unwrap();
									data_stack.push( value );
									flow = flow::Running;
									continue 'frame_loop;
								}
								
								flow::PopSuppressedFlow => {
									suppressed_flows.pop();
								}
							}
						}
						
					} // flow::Returning( value )
					
					flow::Throwing( throwable ) => {
						
						loop {
							
							if flow_points.len() == 0 {
								return result::UncaughtThrowable( throwable );
							}
							
							match flow_points.pop().unwrap() {
								
								flow::StartCatch { instruction: i } => {
									flow = flow::Catching( throwable );
									frame.instruction = i;
									continue 'frame_loop;
								}
								
								flow::StartFinally { instruction: i } => {
									suppressed_flows.push( flow::Throwing( throwable ) );
									flow = flow::Running;
									flow_points.push( flow::PopSuppressedFlow );
									frame.instruction = i;
									continue 'frame_loop;
								}
								
								flow::PopFrame => {
									frame = frames.pop().unwrap();
								}
								
								flow::PopSuppressedFlow => {
									suppressed_flows.pop();
								}
							}
						}
						
					} // flow::Throwing( e )
					
				} // match flow
				
			} // 'flow_loop
			
		} // 'frame_loop
	}
}
