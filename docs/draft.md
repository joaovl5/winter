# Winter Language Spec

## Nix Counterparts

### Operators

#### Number Operators

- `(+ a b ...)`
- `(- a b)`
- `(- a)`
- `(* a b ...)`
- `(/ a b)`
- `(> a b)`
- `(>= a b)`
- `(< a b)`
- `(<= a b)`

#### Boolean Operators

- `(= a b)`
- `(! a)`
- `(!= a b)`
- `(&& a b ...)`
- `(|| a b ...)`
- `(-> a b ...)`

#### Attribute Operators

- `(. a "b")` is the same as `a.$"{b}"` or `a.${b}`
- `(or a.b c)` equivalent to Nix's `a.b or c`
- `(has a b)` equivalent to Nix's `a ? ${b}`

#### String/Path Operators

- `(+ a b ...)`

#### List Operators

- `(++ a b ...)`

#### Map/Attrset Operators

- `(// a b ...)`

#### Function Operators

- `(f a b c)` equivalent to Nix's `f a b c`

##### Pipe Operators

- `(|> a b ...)` equivalent to Nix's `a |> b |> c`
- `(<| a b ...)` equivalent to Nix's `a <| b <| c`

As of November 2026, these operators remain experimental features in
Nix. We can, however, transpile these operators into the
non-experimental counterparts (e.g.: instead of `a |> b`, we simply
just do `b (a)`.) The implications around compatibility between both
approaches were not yet verified, so this is gated behind a compiler
flag for now, as `{pipe-op-tactic "transpile"}`. See [the section on
the `winter.win` file](#winterwin-file) for more info on how to enable
this feature.

### Builtins

## Data Types

### Scalar Data Types

- `nil`
- `bool`
- `num`
- `str`
- `path`

#### String Interpolation

### Compound Data Types

- `list`
- `record`

#### Records

```nix
{
  inherit baz;
  inherit (foo) bar;

  a.b.c = 1;
  d.f = {g.h = 2;};
  g.${username} = null;
}
```

```scheme
{(inherit baz)
 (inherit foo [bar])

 a.b.c 1
 d.f   {g.h 2}
 g.${username} nil}
```

### Compile-time Data Types

- `expr` - s-expression lists, used for macros

## Bindings

### Inherits

### `with`

There is no `with`. You use the improved `inherit` semantics instead
in Winter.

## Functions

## Comments

### Regular comments

Code comments are made by a single `;`, they're single-line.

### Doc-Comments

Documentation comments can be made with `;.`. They are useful for
documentation generators, LSP diagnosis/hover-popups, and so on.

### Form Comments

Similar to Clojure, blocks/expressions can be easily ignored via a
`;_` prefix. This is typically useful when debugging.

## Typing

```scheme
; plain type reference denotes NewType pattern
(type Email (str))
(type Person
      ; curly brace denotes type is "struct-like"
      {(str name)
       (num age)})
(type Span
      {(num pos_start)
       (num pos_close)})
(type Spanned
      (forall T ; generics
       {(Span span)
        (T value)}))
(type MyError
      ; square bracket denotes tagged union - i.e. rust enums
      [(OUT-OF-MEMORY) ; empty type w.o. payload
       (FILE-NOT-FOUND {(str file-name)})])
; empty types like the MyError.OUT-OF-MEMORY will raise an error if
; tried to be compiled down to Nix, this is to prevent unwanted
; behavior of implicitly casting such types to strings and forcing the
; programmer to handle these cases

; instantiation of a type:
(locals my-span (Span {pos_start 1
                       pos_close 2}))

; function without and with type annotations
(fn sum [x y] (+ x y))
(fn (num sum) [(num x) (num y)] (+ x y))
(fn
 (forall T (T multiply) [(T x) (T y)]
  (* x y)))

; referring to identifiers that conflict with keywords like num can be
; done by prefixing an at sign to the identifier. this is useful for
; handling/porting code that requires these names
(locals @num 1) ; compiles to `num = 1`
```

## Meta-programming

### Macros

```scheme
; compile-time forms prepend `!` for clarity
; macro names require a `!` at the end
(!macro infix!
  [infixed]
  (!expr (!second infixed) (!first infixed) (!last infixed)))

; turns:
(infix! (1 + 1))
; into:
(+ 1 1)

; another variation:
(!macro infix!
  [[a op b]] ; destructuring of s-expr list
  (!expr op a b))

; with typing:
(!macro infix!
  [(!expr [a op b])])
```

#### Macro forms

- `!first` - grabs 1, i.e. `car` or `head`
- `!second` - grabs 2nd element
- `!last` - grabs last element
- `!tail` - exp without first, i.e. `cdr`
- `!unpack` - unwraps an `expr` or `list` into an outer scope
- `!gensym` - ensures identifier names won't shadow existing ones
  wherever a macro is expanded - can be simplified by using a hash
  symbol at the end of an identifier like in Clojure or Fennel, e.g.:
  `name#`

## Projects and Builds

### `winter.win` file

The file has some restrictions compared to baseline Winter code, they
are:

- The file is not compiled down to Nix and then evaluated - the
  evaluation is done internally, in a subset of the baseline Winter
  capabilities.
- The guaranteed functionality for this file is not yet settled for
  this specification.
- The file's expression must evaluated to the configuration record
  type or will fail with a type error.

```scheme
(namespace-use (as winter-compiler wc))

(wc/WinterConfig {a})
```

Internally, the `WinterConfig` type is like:

```scheme
(namespace winter-compiler)

;. Schema for the `winter.win` configuration file.
(type WinterConfig
      {(path src-directory)
       ()})

;. Manages how to handle pipe operators when compiling to Nix.
(type PipeOpTactic
      [;. Use the experimental-feature version
       (NATIVE)
       ;. unwind pipelines into nested function calls
       (TRANSPILE)
       ;. use nixpkgs `lib.pipe`
       (NIXPKGS)])

```

### Namespaces

- Namespaces require a `winter.win` file to operate, otherwise will
  fail. This is ellaborated on the [SECTION].
- Namespaces form a directed-acyclic-graph (DAG) from the set of files
  in the project

```scheme
; namespaces are opt-in! to make a namespace out of your file, do:
(namespace my-utils)
; a namespace will only work when your project has a `winter.win` file
; that the compiler will use for building it

```
