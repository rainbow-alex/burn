use error::AnalysisError;
use compile::analysis::{FrameAnalysis, ScopeAnalysis};
use parse::node;
use mem::raw::Raw;

pub struct FindVariableDeclarations {
	frame: Raw<FrameAnalysis>,
	scope: Raw<ScopeAnalysis>,
	pub errors: Vec<AnalysisError>,
}

	impl FindVariableDeclarations {
		
		pub fn new() -> FindVariableDeclarations {
			FindVariableDeclarations {
				frame: Raw::null(),
				scope: Raw::null(),
				errors: Vec::new(),
			}
		}
		
		pub fn analyze_root( &mut self, root: &mut node::Root ) {
			
			self.frame = Raw::new( &root.frame );
			
			root.scope.frame = self.frame;
			self.analyze_statements( &mut root.scope, &mut root.statements );
		}
		
		fn analyze_statements( &mut self, scope: &mut ScopeAnalysis, statements: &mut Vec<Box<node::Statement>> ) {
			
			let containing_scope = self.scope;
			self.scope = Raw::new( scope );
			
			for statement in statements.mut_iter() {
				self.analyze_statement( *statement );
			}
			
			self.scope = containing_scope;
		}
		
		fn analyze_statement( &mut self, statement: &mut node::Statement ) {
			match *statement {
				
				node::If {
					test: ref mut test,
					scope: ref mut if_scope,
					block: ref mut if_block,
					else_if_clauses: ref mut else_if_clauses,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *test );
					self.analyze_statements( if_scope, if_block );
					
					for else_if_clause in else_if_clauses.mut_iter() {
						self.analyze_expression( else_if_clause.test );
						self.analyze_statements( &mut else_if_clause.scope, &mut else_if_clause.block );
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
						}
						None => {}
					};
				}
				
				node::Try {
					scope: ref mut try_scope,
					block: ref mut try_block,
					catch_clauses: ref mut catch_clauses,
					else_clause: ref mut else_clause,
					finally_clause: ref mut finally_clause,
				} => {
					
					self.analyze_statements( try_scope, try_block );
					
					for catch_clause in catch_clauses.mut_iter() {
						
						match catch_clause.type_ {
							Some( ref mut expression ) => {
								self.analyze_expression( *expression );
							}
							None => {}
						}
						
						catch_clause.scope.frame = self.frame;
						
						catch_clause.variable.declared_in = Raw::new( &catch_clause.scope );
						catch_clause.scope.declared.push( Raw::new( &catch_clause.variable ) );
						
						self.analyze_statements( &mut catch_clause.scope, &mut catch_clause.block );
					}
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
						}
						None => {}
					};
					
					match *finally_clause {
						Some( ref mut finally_clause ) => {
							self.analyze_statements( &mut finally_clause.scope, &mut finally_clause.block );
						}
						None => {}
					};
				}
				
				node::While {
					test: ref mut test,
					scope: ref mut while_scope,
					block: ref mut while_block,
					else_clause: ref mut else_clause,
				} => {
					
					self.analyze_expression( *test );
					self.analyze_statements( while_scope, while_block );
					
					match *else_clause {
						Some( ref mut else_clause ) => {
							self.analyze_statements( &mut else_clause.scope, &mut else_clause.block );
						}
						None => {}
					};
				}
				
				node::Let {
					variable: ref mut variable,
					default: ref mut default,
					..
				} => {
					variable.declared_in = self.scope;
					self.scope.get().declared.push( Raw::new( variable ) );
					
					match *default {
						Some( ref mut expression ) => self.analyze_expression( *expression ),
						_ => {}
					}
				}
				
				node::Print {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
				
				node::Return {
					expression: ref mut expression,
				} => {
					match *expression {
						Some( ref mut e ) => self.analyze_expression( *e ),
						None => {},
					}
				}
				
				node::Throw {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
				
				node::Assignment {
					lvalue: ref mut lvalue,
					rvalue: ref mut rvalue,
				} => {
					self.analyze_lvalue( *lvalue );
					self.analyze_expression( *rvalue );
				}
				
				node::ExpressionStatement {
					expression: ref mut expression,
				} => {
					self.analyze_expression( *expression );
				}
			}
		}
		
		fn analyze_expression( &mut self, expression: &mut node::Expression ) {
			
			match *expression {
				ref f @ node::Function {..} => {
					self.scope.get().functions.push( Raw::new( f ) );
				}
				_ => {}
			}
			
			match *expression {
				
				node::And { left: ref mut left, right: ref mut right }
				| node::Or { left: ref mut left, right: ref mut right }
				| node::Is { left: ref mut left, right: ref mut right }
				| node::Union { left: ref mut left, right: ref mut right }
				| node::Addition { left: ref mut left, right: ref mut right }
				| node::Subtraction { left: ref mut left, right: ref mut right }
				| node::Multiplication { left: ref mut left, right: ref mut right }
				| node::Division { left: ref mut left, right: ref mut right }
				=> {
					self.analyze_expression( *left );
					self.analyze_expression( *right );
				}
				
				node::Not { expression: ref mut expression }
				=> {
					self.analyze_expression( *expression );
				}
				
				node::DotAccess {
					expression: ref mut expression,
					..
				} => {
					self.analyze_expression( *expression );
				}
				
				node::ItemAccess {
					expression: ref mut expression,
					key_expression: ref mut key_expression,
				} => {
					self.analyze_expression( *expression );
					self.analyze_expression( *key_expression );
				}
				
				node::Call {
					expression: ref mut expression,
					arguments: ref mut arguments,
				} => {
					self.analyze_expression( *expression );
					for argument in arguments.mut_iter() {
						self.analyze_expression( *argument );
					}
				}
				
				node::Function {
					parameters: ref mut parameters,
					closure: _,
					scope: ref mut scope,
					block: ref mut block,
				} => {
					
					for parameter in parameters.mut_iter() {
						
						match parameter.type_ {
							Some( ref mut expression ) => self.analyze_expression( *expression ),
							None => {},
						}
						
						match parameter.default {
							Some( ref mut expression ) => self.analyze_expression( *expression ),
							None => {},
						}
						
						scope.declared.push( Raw::new( &parameter.variable ) );
						parameter.variable.declared_in = Raw::new( scope );
					}
					
					self.analyze_statements( scope, block );
				}
				
				node::Variable {..}
				| node::Name {..}
				| node::String {..}
				| node::Integer {..}
				| node::Float {..}
				| node::Boolean {..}
				| node::Nothing
				=> {}
			}
		}
		
		fn analyze_lvalue( &mut self, lvalue: &mut node::Lvalue ) {
			match *lvalue {
				
				node::VariableLvalue {..}
				=> {}
			}
		}
	}
