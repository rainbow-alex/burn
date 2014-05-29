use mem::raw::Raw;
use lang::identifier::Identifier;

#[deriving(Show)]
pub enum OpCode {
	
	// Temp
	Print,
	
	// VM commands
	Nop,
	End,
	Fail,
	
	// Scopes, locals, cells
	// PushLocal { pub depth: u32, pub index: u32 },
	// PopLocal { pub depth: u32, pub index: u32 },
	
	// Flow
	PopFlowPoint,
	Jump { pub instruction: uint },
	JumpIfPopFalsy { pub instruction: uint },
	FlowJump { pub n_flow_points: uint, pub instruction: uint },
	
	// Function flow
	Call { pub n_arguments: uint },
	Return,
	
	// Try catch
	PushStartCatchFlowPoint { pub instruction: uint },
	PushStartFinallyFlowPoint { pub instruction: uint },
	Throw,
	CatchOrJump { pub instruction: uint },
	Catch,
	Rethrow,
	StartFinally,
	EndFinally,
	
	// Data stack operations
	Pop,
	
	// Values
	PushFunction { pub index: uint },
	//PushBoundBurnFunction { pub index: uint },
	PushString { pub index: uint },
	PushFloat { pub value: f64 },
	PushInteger { pub value: i64 },
	PushBoolean { pub value: bool },
	PushNothing,
	InlinedModule { pub ptr: Raw<::lang::module::Module> },
	
	// Variables
	StoreLocal { pub index: uint },
	LoadLocal { pub index: uint },
	StoreSharedLocal { pub index: uint },
	LoadSharedLocal { pub index: uint },
	StoreStaticBound { pub index: uint },
	LoadStaticBound { pub index: uint },
	StoreSharedBound { pub index: uint },
	LoadSharedBound { pub index: uint },
	
	// Names
	Use { pub operation: Raw<::lang::module::Use> },
	LoadImplicit { pub name: Identifier },
	
	// Access
	GetProperty { pub name: Identifier },
	SetProperty { pub name: Identifier },
	GetItem,
	
	// Operations
	Is,
	Eq,
	Neq,
	Lt,
	Gt,
	LtEq,
	GtEq,
	Union,
	Add,
	Subtract,
	Multiply,
	Divide,
	Not,
	ShortCircuitAnd,
	ShortCircuitOr,
}
