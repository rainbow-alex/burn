use std::mem;
use lang::value;
use lang::function::Function;
use lang::operations;
use vm::bytecode::opcode;
use vm::result;
use vm::virtual_machine::VirtualMachine;
use vm::run::fiber::Fiber;
use vm::run::{frame, flow, rust};
use vm::run::rust::Operation;
use builtin::burn::{errors, types};

pub fn run( vm: &mut VirtualMachine, mut fiber: Box<Fiber> ) -> result::Result {
	
	'frame_loop: loop {
	
	let rust_result = match fiber.frame.type_ {
		
		frame::Rust( ref mut operation ) => {
			
			let result = match mem::replace( &mut fiber.flow, flow::Running ) {
				flow::Running => Ok( value::Nothing ),
				flow::Returning( v ) => Ok( v ),
				flow::Throwing( v ) => Err( v ),
				_ => fail!(),
			};
			
			Some( operation.run( vm, result ) )
		}
		
		_ => None
	};
	
	// I have to start a new match to end the fiber borrow because of rust#6393
	match rust_result {
		
		Some( result ) => {
			match result {
				
				rust::Ok( value ) => {
					fiber.pop_frame();
					fiber.data_stack.push( value );
					fiber.flow = flow::Running;
					continue 'frame_loop;
				}
				
				rust::Throw( value ) => {
					fiber.pop_frame();
					fiber.flow = flow::Throwing( value );
					continue 'frame_loop;
				}
				
				rust::Burn( frame ) => {
					fiber.flow_points.push( flow::PopFrame { data_stack_len: fiber.data_stack.len() } );
					fiber.push_frame( frame );
					fiber.flow = flow::Running;
					continue 'frame_loop;
				}
				
				_ => fail!()
			}
		}
		
		None => {}
	}
	
	let opcodes = fiber.frame.get_code().opcodes.as_mut_ptr();
	
	'flow_loop: loop {
	match fiber.flow.clone() { // clone because of rust#6393
		
		flow::Running | flow::Catching(..) => {
			
			'instruction_loop: loop {
				
				debug!( {
					println!(
						"VM: running {}/{} ({})",
						fiber.frame.instruction.clone(), // clone because of unidentified rust bug
						fiber.frame.get_code().opcodes.len(),
						fiber.flow_points.len()
					);
				} )
				
				match unsafe { *opcodes.offset( fiber.frame.instruction as int ) } {
					
					// Temporary
					
					opcode::Print => {
						println!( "{}", fiber.data_stack.pop().unwrap().to_string() );
					}
					
					// VM
					
					opcode::Nop => {}
					
					opcode::End => {
						if fiber.frame_stack.len() > 0 {
							fiber.flow = flow::Returning( value::Nothing );
							continue 'flow_loop;
						} else {
							return result::Done;
						}
					}
					
					opcode::ReturnPop => {
						fail!(); // TODO remove this opcode
					}
					
					// Flow
					
					opcode::PopFlowPoint => {
						fiber.flow_points.pop();
					}
					
					opcode::Jump { instruction: i } => {
						fiber.frame.instruction = i;
						continue 'instruction_loop;
					}
					
					opcode::JumpIfPopFalsy { instruction: i } => {
						if ! operations::is_truthy( &fiber.data_stack.pop().unwrap() ) {
							fiber.frame.instruction = i;
							continue 'instruction_loop;
						}
					}
					
					opcode::FlowJump { n_flow_points: n, instruction: i } => {
						fiber.flow = flow::Jumping { n_flow_points: n, instruction: i };
						continue 'flow_loop;
					}
					
					// Functions
					
					opcode::Call { n_arguments: n_arguments } => {
						
						let args_offset = fiber.data_stack.len() - n_arguments;
						let args_ptr = unsafe { fiber.data_stack.as_ptr().offset( args_offset as int ) };
						unsafe { fiber.data_stack.set_len( args_offset ) }
						
						match fiber.data_stack.pop().unwrap() {
							
							value::Function( function ) => {
								
								assert!( n_arguments == 0 ); // TODO
								(args_ptr);
								
								fiber.frame.instruction += 1;
								fiber.flow_points.push( flow::PopFrame { data_stack_len: fiber.data_stack.len() } );
								
								let locals = Vec::new();
								let shared = Vec::new();
								
								let new_frame = frame::Frame {
									type_: frame::Function( function ),
									local_variables: locals,
									shared_local_variables: shared,
									instruction: 0,
								};
								
								let old_frame = mem::replace( &mut fiber.frame, new_frame );
								fiber.frame_stack.push( old_frame );
								
								continue 'frame_loop;
							}
							
							_ => { fail!(); } // TODO
						}
					}
					
					opcode::Return => {
						fiber.flow = flow::Returning( fiber.data_stack.pop().unwrap() );
						continue 'flow_loop;
					}
					
					// Try/Catch
					
					opcode::PushStartCatchFlowPoint { instruction: i } => {
						fiber.flow_points.push( flow::StartCatch { instruction: i } );
					}
					
					opcode::PushStartFinallyFlowPoint { instruction: i } => {
						fiber.flow_points.push( flow::StartFinally { instruction: i } );
					}
					
					opcode::Throw => {
						
						let throwable = fiber.data_stack.pop().unwrap();
						
						if types::is_throwable( &throwable ) {
							fiber.flow = flow::Throwing( throwable );
						} else {
							let message = format!( "{} is not Throwable.", throwable.repr() );
							let error = errors::create_type_error( message );
							fiber.flow = flow::Throwing( error );
						}
						
						continue 'flow_loop;
					}
					
					opcode::CatchOrJump { instruction: i } => {
						
						let throwable = match fiber.flow {
							flow::Catching( ref t ) => t.clone(), // ref+clone because of rust#6393
							_ => fail!(),
						};
						
						let type_ = fiber.data_stack.pop().unwrap();
						let result = operations::is( &throwable, &type_ );
						
						match result {
							Ok( true ) => {
								*fiber.frame.local_variables.get_mut( 0 ) = throwable;
								fiber.flow = flow::Running;
							},
							Ok( false ) => {
								fiber.frame.instruction = i;
								fiber.flow = flow::Catching( throwable );
								continue 'instruction_loop;
							}
							Err( e ) => {
								fiber.flow = flow::Throwing( e );
								continue 'flow_loop;
							}
						}
					}
					
					opcode::Catch => {
						
						let throwable = match fiber.flow {
							flow::Catching( ref t ) => t.clone(), // ref+clone because of rust#6393
							_ => fail!(),
						};
						
						*fiber.frame.local_variables.get_mut( 0 ) = throwable;
						fiber.flow = flow::Running;
					}
					
					opcode::Rethrow => {
						
						fiber.flow = match fiber.flow {
							flow::Catching( ref e ) => flow::Throwing( e.clone() ), // ref+clone because of rust#6393
							_ => fail!(),
						};
						
						continue 'flow_loop;
					}
					
					opcode::StartFinally => {
						fiber.flow_points.pop();
						fiber.suppressed_flows.push( flow::Running );
						fiber.flow_points.push( flow::PopSuppressedFlow );
					}
					
					opcode::EndFinally => {
						fiber.flow_points.pop();
						fiber.flow = fiber.suppressed_flows.pop().unwrap();
						match fiber.flow {
							flow::Running => {},
							_ => continue 'flow_loop,
						};
					}
					
					// Data stack operations
					
					opcode::Pop => {
						fiber.data_stack.pop();
					}
					
					// Values
					
					opcode::PushFunction { index: i } => {
						
						fiber.data_stack.push(
							value::Function(
								vm.functions.register(
									Function::new(
										fiber.frame.get_code().functions.get( i ).clone()
									)
								)
							)
						);
					}
					
					opcode::PushString { index: i } => {
						fiber.data_stack.push( value::String( fiber.frame.get_code().strings.get( i ).clone() ) );
					}
					
					opcode::PushFloat { value: f } => {
						fiber.data_stack.push( value::Float( f ) );
					}
					
					opcode::PushInteger { value: i } => {
						fiber.data_stack.push( value::Integer( i ) );
					}
					
					opcode::PushBoolean { value: b } => {
						fiber.data_stack.push( value::Boolean( b ) );
					}
					
					opcode::PushNothing => {
						fiber.data_stack.push( value::Nothing );
					}
					
					opcode::InlinedModule { ptr: ptr } => {
						fiber.data_stack.push( value::Module( ptr ) );
					}
					
					// Variables
					
					opcode::StoreLocal { index: i } => {
						*fiber.frame.local_variables.get_mut( i ) = fiber.data_stack.pop().unwrap();
					}
					
					opcode::LoadLocal { index: i } => {
						fiber.data_stack.push( fiber.frame.local_variables.get( i ).clone() );
					}
					
					opcode::StoreSharedLocal { index: i } => {
						*fiber.frame.shared_local_variables.get( i ).get() = fiber.data_stack.pop().unwrap();
					}
					
					opcode::LoadSharedLocal { index: i } => {
						fiber.data_stack.push( fiber.frame.shared_local_variables.get( i ).get().clone() );
					}
					
					opcode::StoreStaticBound { index: i } => {
						*fiber.frame.get_closure().static_bound_variables.get_mut( i ) = fiber.data_stack.pop().unwrap();
					}
					
					opcode::LoadStaticBound { index: i } => {
						fiber.data_stack.push( fiber.frame.get_closure().static_bound_variables.get( i ).clone() );
					}
					
					opcode::StoreSharedBound { index: i } => {
						*fiber.frame.get_closure().shared_bound_variables.get( i ).get() = fiber.data_stack.pop().unwrap();
					}
					
					opcode::LoadSharedBound { index: i } => {
						fiber.data_stack.push( fiber.frame.get_closure().shared_bound_variables.get( i ).get().clone() );
					}
					
					// Names
					
					opcode::Use { operation: operation } => {
						let operation = unsafe { operation.get_box() };
						unsafe { *opcodes.offset( fiber.frame.instruction as int ) = opcode::Nop; }
						
						fiber.frame.instruction += 1;
						fiber.push_frame( frame::Frame {
							type_: frame::Rust( operation as Box<rust::Operation> ),
							local_variables: Vec::new(),
							shared_local_variables: Vec::new(),
							instruction: 0,
						} );
						continue 'frame_loop;
					}
					
					opcode::LoadImplicit { name: name } => {
						match vm.implicit.get().find_id( name ) {
							Ok( value ) => {
								fiber.data_stack.push( value.clone() );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						}
					}
					
					// Access
					
					opcode::GetProperty { name: name } => {
						let left = fiber.data_stack.pop().unwrap();
						match operations::get_property( &left, name ) {
							Ok( value ) => {
								fiber.data_stack.push( value );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						}
					}
					
					opcode::SetProperty { name: name } => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::set_property( &left, name, &right ) {
							Ok(..) => {},
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						}
					}
					
					// Operators
					
					opcode::Is => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::is( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Eq => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::eq( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Neq => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::neq( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Lt => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::lt( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Gt => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::gt( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::LtEq => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::lt_eq( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::GtEq => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::gt_eq( &left, &right ) {
							Ok( result ) => {
								fiber.data_stack.push( value::Boolean( result ) );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Union => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::union( left, right ) {
							Ok( result ) => {
								fiber.data_stack.push( result );
							}
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Add => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::add( &left, &right ) {
							Ok( result ) => fiber.data_stack.push( result ),
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
					opcode::Subtract => {
						let right = fiber.data_stack.pop().unwrap();
						let left = fiber.data_stack.pop().unwrap();
						match operations::subtract( &left, &right ) {
							Ok( result ) => fiber.data_stack.push( result ),
							Err( err ) => {
								fiber.flow = flow::Throwing( err );
								continue 'flow_loop;
							}
						};
					}
					
				} // match opcode
				
				fiber.frame.instruction += 1;
				
			} // 'instruction_loop
			
		} // flow::Running | flow::Catching
		
		flow::Jumping { n_flow_points: mut n_flow_points, instruction: instruction } => {
			
			while n_flow_points > 0 {
				
				match fiber.flow_points.pop().unwrap() {
					
					flow::StartCatch {..} => {
						// ignored, there is no throwable that needs to be caught
					}
					
					flow::StartFinally { instruction: i } => {
						fiber.suppressed_flows.push( flow::Jumping { n_flow_points: n_flow_points, instruction: instruction } );
						fiber.flow = flow::Running;
						fiber.flow_points.push( flow::PopSuppressedFlow );
						fiber.frame.instruction = i;
						continue 'flow_loop;
					}
					
					flow::PopFrame {..}  => fail!(),
					
					flow::PopSuppressedFlow => {
						fiber.suppressed_flows.pop();
					}
				}
				
				n_flow_points -= 1;
			}
			
		} // flow::Jumping( e )
		
		flow::Returning( value ) => {
			
			loop {
				match fiber.flow_points.pop().unwrap() {
					
					flow::StartCatch {..} => {
						// ignored, there is no throwable that needs to be caught
					}
					
					flow::StartFinally { instruction: i } => {
						fiber.suppressed_flows.push( flow::Returning( value ) );
						fiber.flow = flow::Running;
						fiber.flow_points.push( flow::PopSuppressedFlow );
						fiber.frame.instruction = i;
						continue 'flow_loop;
					}
					
					flow::PopFrame { data_stack_len: n } => {
						fiber.pop_frame();
						fiber.data_stack.truncate( n );
						fiber.data_stack.push( value );
						fiber.flow = flow::Running;
						continue 'frame_loop;
					}
					
					flow::PopSuppressedFlow => {
						fiber.suppressed_flows.pop();
					}
				}
			}
			
		} // flow::Returning( value )
		
		flow::Throwing( throwable ) => {
			
			loop {
				
				if fiber.flow_points.len() == 0 {
					return result::UncaughtThrowable( throwable );
				}
				
				match fiber.flow_points.pop().unwrap() {
					
					flow::StartCatch { instruction: i } => {
						fiber.flow = flow::Catching( throwable );
						fiber.frame.instruction = i;
						continue 'frame_loop;
					}
					
					flow::StartFinally { instruction: i } => {
						fiber.suppressed_flows.push( flow::Throwing( throwable ) );
						fiber.flow = flow::Running;
						fiber.flow_points.push( flow::PopSuppressedFlow );
						fiber.frame.instruction = i;
						continue 'frame_loop;
					}
					
					flow::PopFrame { data_stack_len: n } => {
						fiber.pop_frame();
						fiber.data_stack.truncate( n );
						fiber.frame = fiber.frame_stack.pop().unwrap();
						continue 'frame_loop;
					}
					
					flow::PopSuppressedFlow => {
						fiber.suppressed_flows.pop();
					}
				}
			}
			
		} // flow::Throwing( e )
		
	} // match flow
	} // 'flow_start
	} // 'frame_start
}
