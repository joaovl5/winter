# The Winter Language Spec

Sole language focus as of now is compiling into Nix.

## Operators

Operators are the same from Nix.

## Data Types

### Scalar Data Types

- Numbers (`num`)
  - No distinction between floats, integers, and so on
  - `123` - `1.24`
- Booleans (`bool`)
  - `true` - `false`
- Nil (`nil`)
  - Compiles to Nix's `null`
  - `nil`
- Strings (`str`)
  - May be interpolated, compiling down to Nix's equivalent string
    interpolation syntax: `"1+1 is ${1+1}"`
    - Interpolated with the syntax: `$"1+1 is {(+ 1 1)}"`
    - Interpolation can be canceled by wrapping the curly braces, as
      in `1+1 is in the set {{2}}`
    - Interpolated values should resolve to strings and won't be
      implicitly cast to strings by the Winter compiler.

### Compound Data Types

- List (`list`)
  - Compiles down to a Nix list
  - Wrapped in square brackets like `[1 2 3 4]`
- Map (`map`)
  - Compiles down to a Nix attribute set
  - Wrapped in curly braces
  - Setting nested keys can use the regular `.` syntax from Nix
  - As with Nix, keys remain implicitly strings, by default, but by
    preceding a key with the `!` operator, it will allow the key to be
    a non-string type, and compilation to Nix will auto-handle the
    string conversion (and fail if not possible to convert automatically)
    - This is useful for mapping non-string keys to other values, like
      for custom types, enums, and so on.
  - As with Nix, keys are still implicitly strings - but different
    from Nix, that doesn't need to be the case! They eventually will
    need to be compiled down to strings, but they aren't represented
    as such and can be declared types, integers, and so on.
  - For yielding an string attrset path from non-string forms, use
    the helper `!` syntax as seen below:

    ```text
    {this-is-one 1
     this-is-two 2
     here.are.nested.keys "wow"
     the.sum.of.$"{(+ 1 1)}.and.one" 3
     "quoted"."strings"."also work" {}
     ! SomeEnumIHave.SomeEnumValue "bla bla bla"}
    ```

## Variable bindings

### Let

`let` works similarly as in the `let ... in ...` from Nix, by taking
a list of variable bindings and injecting them into some 'body'
expression, which in the example below is an empty map.

```text
(let
  [x "value"
   y 1.2]
 {})
```

### Locals

`locals` is similar to a `let` call, but references to its bindings
can be done within the same scope, without needing to wrap all
consuming expressions inside the `let` call itself:

```text
(locals
  x "value"
  y "value")
{}
```

Under the hood, `locals` will be compiled down to a `let` call in Nix
based on the scope.

### Inherit

- Both for `let` and `locals`, an `inherit` syntax can be used as seen
below.
- It may also be used without being inside the bindings of a `let`
  call or inside a `locals` call:
  - If used inside a map, it will auto-resolve to an `inherit` inside
    that map (attrset)
  - If used elsewhere, will be resolved to a `locals` call and, in
    turn, a `let` call
- The syntax is inverted from Nix in the sense that the namespace
  we're inherting a value from is *not* wrapped in parentheses, but
  its values being inherited are
  - The way Winter determines to inherit a namespace's objects or the
    namespace itself - so differing between what would be a
    `inherit (abc) xyz` and an `inherit abc xyz` - is the usage of
    parentheses as seen below (using parentheses will imply the items
    within the form wrapped in parentheses will be inherited from the
    namespace identified by the preceding form)

```text
(inherit
  lib (getExe)
  pkgs (steam)
  mylib ; will inherit mylib itself
  )
```

## Functions

Nix lambdas/functions follow the form below:

```text
(fn [x y]
  (+ x y))

; can also include a name
; which will get auto-resolved to a local variable based on scope
; like a `locals` call
(fn sum-values [x y]
  (+ x y))

; unpacking map (attrset) arguments also works here
; with the same `?` coallescing syntax from nix
; also ellipsis work the same with nix to allow for extra unused keys
(fn make-potion
  [cauldron
   {use-element-x ? false
    use-uranium ? true
    ...}]
  (do-stuff))

; if map is only argument to `fn`, square brackets can be omitted:
(fn make-potion
  {use-uranium ? false
   use-element-y ? true}
    (do-stuff))

; naming an unpacked map is done via angle-brackets like below:
(fn make-pizza
  [eggs
   dough
   <extra-ingredients {use-cheddar ? true
                       use-cheetos ? false
                       ...}>]
  (do-pizza))
```

May also include type annotations (more on those at [SECTION]):

```text
(fn sum-values@num [x@num y@num]
  (+ x y))
```

## Comments

### Regular Comments

Characters in a line that are preceded by `;` are considered comments,
unless the semi-colon is inside a string.

```text
; this is a comment
"; this is not a comment"
```

### Documentation Comments

>[!WARNING]
>This section of the specification is work-in-progress.

Documentation comments, or simply doc-comments, are a special comment
type will be compiled-down into an appropriately-formatted
documentation comment depending on the format of choice. In Winter, it
will a singular structure and validation, and its compiler will compile
such comments into a chosen documentation standard, with the default
being [nixdoc](https://github.com/nix-community/nixdoc).

Type descriptions are notably excluded from the syntax for these
comments because the Winter compiler will auto-resolve them from given
type annotations to the code.

The syntax for doc-comments is as follows:

```text
{;=
   desc: This function adds two numbers
   sample:
     (add 4 5) 9
   args:
     a The first number
     b The second number
 ;=
 (fn add@num [a@num b@num]
   (+ a b))}
```

- all fields aside from `desc` are optional
- each line item in `sample` will take a form depending on the
  documented symbol:
  - for a function, it will expect an application of said function,
    and the evaluated result
- each line item in `args` is expected to be an argument name followed
  by its description

Compiles to:

````nix
{
  /**
    This function adds two numbers

    # Example

    ```nix
    add 4 5
    =>
    9
    ```

    # Type

    ```
    add :: Number -> Number -> Number
    ```

    # Arguments

    a
    : The first number

    b
    : The second number

  */
  add = a: b: a + b;
}
````

## Typing

### Type Annotations

#### Scalars

Refer to the section of [Scalar Data Types](#scalar-data-types) for
checking the type identifiers of all scalar types.

```text
(fn add@num [x@num y@num]
  (+ x y))
```

The additional type `any` exists for denoting *any* type.

#### Compound

##### Lists

*(`list@T` where T is some type)*

```text
(fn average@num [numbers@list@num])
```

##### Maps

*(`map@(K V)` where K and V are types for the keys and values of the
map respectively)*

```text
(fn tallest_person@str [people_map@map@(str num)])
```

#### Functions

Functions type annotations take the form of `fn@(X Y)` where `X` is
a list of types for every positional parameter of the function, and `Y`
is the return type.

```text
(fn call_a_file_deleter
  [; function that takes a string (file to delete)
   ; and a boolean for whether to force deletion if its a directory
   file_deleter@fn@((str bool) nil)
   file_to_delete@str]
  (file_deleter file_to_delete true))
```

#### Unions

Unions may be done with a `@(| X Y ...)` syntax where it takes an
arbitrary amount of types (denoted by the ellipsis) to make an union
out of.

```text
; deletes either a file or a list of files
(fn delete_file@nil [files@(| str list@str)])
```

#### Intersections (Combined Types)

Intersections combine a new type out of two or more types, by
combining their fields. Notably this will only work with *compound*
types that are *declared* (see below in [Declared
Types](#declared-types) for more info on the topic)

The syntax is the same as unions, but using the `&` syntax.

```text
(type Identity
  id@num
  name@str)

(type Contact
  email@str
  phone@str)

(type Employee (& Identity Contact))
```

### Declared Types

Nix won't see these types, so they're only compile-time assurances,
akin to Typescript. The Winter compiler will turn instantiated types
into plain maps.

#### `type` syntax

```text
(type Person
  name@str
  age@num)
```

#### `enum` syntax

Maps identifiers to typed values

```text
(enum FileType@str
  PNG "png"
  JPGEG "jpeg")

(enum FileSizes&num
  KILO 1024
  MEGA (** 1024 2)
  GIGA (** 1024 3))
```

#### Generics

>[!WARNING]
>This section of the specification is work-in-progress.

```text
; a person containing a secret value of type T
(type Person@T
  name@str
  age@num
  secret@T)

; a function with generics
(fn@T multiply@T [x@T y@T]
  (* x y))

; a function with multiple generic types
(fn@(X, Y) bla_bla@(| X Y) [x@(| X Y) y@(| X Y)])
```

### Macros

>[!CAUTION]
>This section of the specification is NOT DONE AT ALL.

- Macros should feel like regular code that is done at compile-time.

```text
(macro module! [name raw_opts body]
  (fn handle_raw_opt [x]
    (locals
      key (first x)
      value (rest v)
      opt_fn_name (first value)
      opt_fn_call (rest value)
      opt_fn (. `o opt_fn_name))
    {key `(opt_fn ,(unpack opt_fn_call))})
  (locals
    opts (transform raw_opts handle_raw_opt))
  `(fn [_]
    {den.aspects.server.nixos
     (fn [mylib config inputs pkgs lib]
      (locals
        my (mylib.use config)
        o my.options
        s my.secrets
        t lib.types)
      (o.module
        ,name
        ,opts
        (fn [opts]
          ,body)))}))

; used like:
(module!
  "unit.octodns"
  {enable (toggle "Enable OctoDNS" false)}
  (o.when opts.enable {systemd.services.octodns-sync {}}))
```

- `transform` - iterates a transformer function `f(x)` over
  a sequence, where `x` will be:
  - for maps, a sequence like `(k v)`
  - for lists, plainly a value of the list `v`
- `first` - roughly equivalent to `car` - takes first form of a sequence
  - for `(k v)` will take `k`
  - for `["x" "z" "w"]` will take `"x"`
  - for `{x 0 y 1 z 2}` will take `(x 0)`
- `rest` - roughly equivalent to `cdr` - yields whatever is left of
  sequence without the result of `first`, still in the form of that
  type of sequence, otherwise returning `nil`
- `unpack` - unpacks elements of a sequence into an outer form
  - `["x" "y" (unpack z)]` with `z` being `["z" "w" "u"]` yields:
    `["x" "y" "z" "w" "u"]`
  - `(a b c (unpack z))` yields `(a b c "z" "w" "u")`
  - for maps it will expect unpacked expression to be a sequence of
    sequences (which a map already is)
    - `{x 0 y 1 (unpack z)}` and `z` = `{z 2 w 3 u 4}` yields
      `{x 0 y 1 z 2 w 3 u 4}`
    - also works if `z` is of form `((z 2) (w 3) (u 4))` or is a list
      like `[["z" 2] ["w" 3] ["u" 4]]`

### Requirements and Namespaces System

>[!CAUTION]
>This section of the specification is VERY MUCH NOT DONE AT ALL.

- Requirements can form a directed acyclic graph
- Namespaces are declared at the top of files with a `namespace` call
  - A file cannot have multiple namespace calls
- Namespaces don't care which directory they're at, and many files
  across directories can 'contribute' to a namespace, with conflicts
  being reported at compile-time.
- Requirements are gathered on a node-basis, where they're consumed,
  for checking if there are circular imports.
- Exported symbols through the namespaces are solely top-level ones.
  - If the last symbol of a file evaluates to a function, that
    function can be named for the symbol to be exported and used
    elsewhere through the namespace.

```text
(namespace common)
(macro module! [...]
  ...)
```

```text
(requires
  common (module!)) ; like the `inherit` syntax
(namespace units.octodns-sync)

(module!
  "unit.octodns"
  {enable (toggle "Enable OctoDNS" false)}
  (o.when opts.enable {systemd.services.octodns-sync {}}))
```
