% Burn - Language specification

About
====

This document tries to describe Burn programs lexically, gramatically and semantically.
I try to be clear and precise, without getting too formal.
Since this is probably the only documentation for the foreseeable future, it also includes examples,
hints, opinions, conventions and other things that aren't strictly part of a language specification document.

Burn is still under development. Everything here is subject to change.
A lot of things aren't implemented yet. Some things are implemented, but not documented.

My name is Alex. If you have questions or comments, send an e-mail to <alex.deleyn@gmail.com>.



Burn in a nutshell
================

Burn intends to be a light-weight, general-purpose programming language.
Here are some of its properties you're likely to know from existing languages:

* Newlines separate statements. Curly braces are mandatory.
* Block scoping.
* Values have types, variables don't. Values are not implicitely coerced.
* Functions are first-class values and can have free variables (lexical closure).
* Object-oriented programming with multiple inheritance. Classes are first-class values too, and support lexical closure.
* Memory is garbage-collected.
* Concurrency is achieved through fibers.
* It compiles to bytecode at runtime. This bytecode is (to some small degree) self-optimizing.

\

Of course there's more to Burn than that. Here are some of the twists that really set Burn apart:

\

**Typing**

Burn's types are more like mathematical sets than the type systems you are probably familiar with.
Think Venn diagrams rather than family trees.
It has the usual types like `Integer` or `String`, but also `Positive` or `Empty`.

There are a lot of types in the standard library, but you can create your own as well.
Types are first-class values and can be combined and created ad hoc:

```
let $Real = Integer | Float
let $NonEmptyString = String + not Empty
let $HttpUrl = String.starting_with( "http://" )
let $CustomType = function( $value ) { return <boolean expression> }
```

\

**Object capabilities**

Burn follows an object-capability model.
Capabilities are functions or objects that can do things with side-effects (e.g. read a file or print to stdout).
All capabilities are given to the program entry point (i.e. `main`), and it is then responsible for passing them to other parts of the program.
This is basically nothing more than language-enforced dependency injection.

* You can, at any time, safely import a library or load a script. It can only have side-effects if and when you give it the required capabilities.
* An untrusted Burn program can be run with reduced capabilities, or by lazily prompting them from the user.
* You can safely embed Burn without cutting off access to third party libraries by simply giving it reduced capabilities.
* Burn programs are probably more likely to be reusable and testable.



Lexical structure
=================

## Comments

Burn supports block and line comments.

<div class="side_by_side"><div>

Block comments start with a `/` followed by one or more `*`.
They end with the exact same amount of stars followed by a `/`, or at the end of the file.

</div><div>

```
/* One block comment */

/** Another **/

/*	I don't end here: **/
	I end here: */

/*
	I end at the end of the file...
```

</div></div>

<div class="side_by_side"><div>

Block comments can't actually be nested.
One block comment can, incidentally, contain another if it has a different amount of stars.

</div><div>

```
/**
	Block comment containing
	/* Another block comment */
**/
```

</div></div>

<div class="side_by_side"><div>

Line comments start with `//` and end at the first newline.

</div><div>

```
// Line comment
print "Hello" // Be polite!
```

</div></div>

<div class="note">
There are no doc comments or attributes at this time.
A generalized annotation syntax is in the works.
</div>

## Whitespace

Spaces and tabs are not significant, besides separating tokens.

## Newlines

Newlines *are* significant. They separate statements:

```
statement
if expression {
	statement
	statement
}
```

This makes burn code easy to read and write.
Sometimes you might want to use newlines to break long expressions and constructs, so they are ignored...

* After binary operators.
* Within parenthesized expressions.
* At the start or end of blocks.
* After keywords, until the end of the expression or construct, but not within their blocks.

## Symbols

```grammar
{ } ( ) [ ]
, -> .
< > == != <= >=
+ - * / % |
+= -= *= /= %=
```

## Keywords

Keywords have special meaning and can't be used where an identifier is expected.

```grammar
and
catch class
else extends
false finally for function
if in is
let
new not nothing
or
return
this throw true try
while
```

## Identifiers

<div class="side_by_side"><div>

Identifiers are composed of one or more letters, digits, `_`, `:` or `!`.
The first character can not be a digit. Any other combination is allowed.

Identifiers are case-sensitive.

`:` should be used sparingly. `!` should be used *especially sparingly*.

</div><div>

```grammar
[A-Za-z_:!][A-Za-z0-9_:!]*
```

</div></div>

## Variables

<div class="side_by_side"><div>

Variables always start with a `$` and are followed by one or more letters, digits, `_`, `:` or `!`.

</div><div>

```grammar
\$[A-Za-z_:!]+
```

</div></div>

## Literals


Statements
==========

## Simple statements

```grammar
lvalue :=
	variable
	| dot_expression
	| item_expression
	| lvalue_tuple

lvalue_tuple := `(` ( lvalue `,` )+ [ lvalue ] `)`
```

### Assignment

#### Augmented assignment operators

### Expression statement

### Import statement








## Control flow statements

```grammar
if_statement :=
	`if` expression block
	( `else` `if` expression block )*
	[ `else` block ]

while_statement :=
	`while` expression block
	[ `else` block ]

for_in_statement :=
	`for` lvalue `in` expression block
	[ `else` block ]
```

### If statement

### While statement

### For-in statement

### Break statement

### Continue statement







## Throwing flow

```grammar
try_statement :=
	`try` block
	( `catch` `(` [ type ] variable `)` block )*
	[ `else` block ]
	[ `finally` block ]

throw_statement := `throw` expression
```

### Try statement

### Throw statement




# Expressions

## Simple expressions

```grammar
simple_expression := access_expression

access_expression := atom_expression | dot_expression | item_expression | call

dot_expression := access_expression `.` identifier

item_expression := access_expression `[` expression `]`

call := access_expression `(` expression_list `)`

atom_expression :=
	function
	| class
	| tuple
	| parenthesized
	| variable
	| identifier
	| literal

tuple := `(` ( expression `,` )+ [ expression ] `)`

parenthesized := `(` expression `)`

literal :=
	string_literal
	| integer_literal
	| float_literal
	| `true` | `false`
	| `nothing`
```

### Item access

### Dot access

### Calling

#### Keyword arguments

### Variables

### Names

### Literals




## Compound expressions

```grammar
compound_expression := logic_expression

logic_expression :=
	not_expression ( `and` not_expression )*
	| not_expression ( `or` not_expression )*

not_expression :=
	is_expression
	| `not` is_expression

is_expression :=
	union_expression
	| union_expression `is` union_expression

union_expression :=
	add_expression
	| union_expression `|` add_expression

add_expression :=
	mul_expression
	| add_expression `+` mul_expression
	| add_expression `-` mul_expression

mul_expression :=
	simple_expression
	| mul_expression `*` simple_expression
	| mul_expression `/` simple_expression
	| mul_expression `%` simple_expression
```

Note that the logical operators `and` and `or` are not expressed recursively.
You can't combine these operators without making precedence explicit through parentheses.

### Addition

The `+` operator is used for addition, string concatenation and taking the intersection of types:

	<Integer> + <Integer> -> <Integer>
	<Integer> + <Float> -> <Float>
	<Float> + <Integer> -> <Float>
	<Float> + <Float> -> <Float>
	
	<String> + <String> -> <String>
	
	<Type> + <Type> -> <Type>

### Subtraction

### Multiplication

### Division

### Modulo

### Union





## Functions

```grammar
function := `function` `(` [ argument_list ] `)` [ `->` type ] block
argument_list := argument ( ',' argument )*
argument := [ type ] variable [ `=` expression ]
```






## Classes

```grammar
class := `class` `{` [ class_items ] `}`
class_items := class_item ( newline class_item )*
class_item := property | method
```







Core semantics
==============

## Scoping

## Type system

## Intrinsics

## Object-oriented programming

## Modules and importing







Execution model
===============

## Fibers

## Memory management







Extending/embedding
===================

## Embedding burn in your application

Rust's ownership and lifetime semantics make extending and embedding burn very easy. Refcounting and
garbage-collecting happens automatically. The VM is inherently unsafe, but this is easily dealt with:

* No `Gc` or `Rc` pointers should outlive the virtual machine that created them.
Any `Gc` pointers still alive will segfault when used.
`Rc` pointers will still point to live data, but any `Gc` pointer contained within them will be invalid.
* No `Gc` or `Rc` pointers should be stored outside the VM.
Doing so is likely to wrongly get their contents garbage-collected.
Consider them valid only until the next garbage collection.

<div class="note">
Externally storable variants of these pointers are planned.
</div>
