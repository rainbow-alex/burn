**UPDATE:** Work on burn is ongoing at [rainbow-alex/burn.js](http://www.github.com/rainbow-alex/burn.js).

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



Of course there's more to Burn than that. Here are some of the twists that really set Burn apart:



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



**Object capabilities**

Burn follows an object-capability model.
Capabilities are functions or objects that can do things with side-effects (e.g. read a file or print to stdout).
All capabilities are given to the program entry point (i.e. `main`), and it is then responsible for passing them to other parts of the program.
This is basically nothing more than language-enforced dependency injection.

* You can, at any time, safely import a library or load a script. It can only have side-effects if and when you give it the required capabilities.
* An untrusted Burn program can be run with reduced capabilities, or by lazily prompting them from the user.
* You can safely embed Burn without cutting off access to third party libraries by simply giving it reduced capabilities.
* Burn programs are probably more likely to be reusable and testable.
