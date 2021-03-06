use mem::raw::Raw;
use lang::identifier::Identifier;

pub enum OpCode {
	
	// Temp
	Print,
	ToString,
	
	// VM commands
	Nop,
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
	TypeCheckLocal { pub index: uint },
	TypeCheckSharedLocal { pub index: uint },
	Return,
	ReturnNothing,
	
	// Try catch
	PushStartCatchFlowPoint { pub instruction: uint },
	PushStartFinallyFlowPoint { pub instruction: uint },
	Throw,
	ThrownIs,
	CatchLocalOrJump { pub storage_index: uint, pub instruction: uint },
	CatchSharedLocalOrJump { pub storage_index: uint, pub instruction: uint },
	CatchLocal { pub storage_index: uint },
	CatchSharedLocal { pub storage_index: uint },
	Rethrow,
	StartFinally,
	EndFinally,
	
	// Data stack operations
	Pop,
	
	// Values
	PushFunction { pub index: uint },
	PushString { pub index: uint },
	PushFloat { pub value: f64 },
	PushInteger { pub value: i64 },
	PushBoolean { pub value: bool },
	PushNothing,
	InlinedModule { pub ptr: Raw<::lang::module::Module> },
	
	// Variables
	StoreLocal( uint ),
	LoadLocal( uint ),
	InitializeSharedLocal( uint ),
	StoreSharedLocal( uint ),
	LoadSharedLocal( uint ),
	StoreStaticBound( uint ),
	LoadStaticBound( uint ),
	StoreSharedBound( uint ),
	LoadSharedBound( uint ),
	
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
