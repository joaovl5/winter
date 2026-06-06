# The Winter Language Spec

>[!IMPORTANT]
>Sole language target as of now is compiling into Nix.

## Operators

### Number Operators

- `(+ a b ...)` sums two or more numbers
- `(- a b)` subtracts `b` from `a`
- `-a` inverts sign of number (parentheses not needed)
- `(* a b ...)` multiplies two or more numbers
- `(/ a b)` divides like `a / b`
- `(> a b)` like `a > b`
- `(< a b)` like `a < b`
- `(>= a b)` like `a >= b`
- `(<= a b)` like `a <= b`

### Boolean Operators

- `(all a b ...)` like `a == b` - takes two or more arguments
- `(any a b ...)` like `a || b` - takes two or more arguments
- `(not= a b)` like `a != b`
- `(not a)` like `!a`
- `(thus a b ...)` like `a -> b -> ...` - takes two or more arguments
  - This is Nix's logical implication operator, where `a -> b` means:
    if `A` is true, then `B` must also be true, if `A` is false just
    return true - it is equivalent to the pseudocode below:

    ```clojure
    if A:
      if B:
        return true
      else:
        return false
    return true
    ```

### Attribute Operators

- `(. a b)` or, if not dynamic, simply `a.b` is fine
- `(or (. a b) (. c d))` equivalent to Nix's `a.b or c.d`
- `(? a b)` like `a ? b` (where `a` is a map/attrset and `b` is an
  attrpath)

### String/Path Operators

`(+ a b ...)` concatenates two or more strings and/or paths

### List Operators

- `(++ a b ...)` concatenates two or more lists

### Map/Attrset Operators

- `(// a b ...)` updates two or more maps/attrsets

### Function Operators

- `(a x y z)` applies a function `a` with arguments `x`, `y`, `z`
  equivalent to `a x y z` in Nix

#### Pipe Operators

>[!WARNING]
>the function operators below, known as *pipe operators* are not included by
>default in the Nix language for most evaluators, and thus if you mean to use
>them you should certify they're enabled as an experimental option to avoid
>issues, follow
>[this link](https://nix.dev/manual/nix/2.26/development/experimental-features#xp-feature-pipe-operators)
>for more info.

- `(|> a b ...)` - takes two or more functions - like `a |> b`
- `(<| a b ...)` - takes two or more functions - like `a <| b`

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
    - Interpolated values will be passed to a `builtins.toString`
      call, thus they get implicitly converted - this will trigger
      eval-time errors for non-coercible types such as attribute-sets.
- Paths (`path`)
  - Differently from Nix, Paths are similar to strings, with a `p`
    prefix, like `p"./config"` compiling into `./config` - thus paths
    must still have quotes.
  - Interpolation might be done by also prefixing `$` before the
    string, but the required order for this must be `p$`
    - so `p$"./config/{dynamic}"` is valid
    - and `$p"./config/{dynamic}"` is not

### Compound Data Types

- List (`list`)
  - Compiles down to a Nix list
  - Wrapped in square brackets like `[1 2 3 4]`
- Map (`map`)
  - Compiles down to a Nix attribute set
  - Wrapped in curly braces
  - Setting nested keys can use the regular `.` syntax from Nix
  - As with Nix, keys are still implicitly strings - but different
    from Nix, that doesn't need to be the case! They eventually will
    need to be compiled down to strings, but they aren't represented
    as such and can be declared types, integers, and so on.
    - This is useful for mapping non-string keys to other values, like
  - For yielding an string attrset path from non-string forms, use
    the helper `!` syntax as seen below:

    ```clojure
    {this-is-one 1
     this-is-two 2
     here.are.nested.keys "wow"
     the.sum.of.$"{(+ 1 1)}.and.one" 3
     "quoted"."strings"."also work" {}
     ! SomeEnumIHave.SomeEnumValue "bla bla bla"}
    ```

  - Thus, with the `!` syntax, keys may be of any value that can
    become a string, and will only actually become strings once they
    get compiled-down into Nix code
    - The `!` key is not necessarily equivalent to a Nix dynamic string attr
      like `${value}` - if this is needed, use string interpolations
      like `$"{value}"`
    - This is because the `!` will attempt handling an expression at
      compile-time, and thus any evaluation-time logic required cannot
      be garanteed by using it
  - A compilation error occurs if a type cannot be converted into
    a string, or if a converted string conflicts with an
    already-existing key at the map.
  - The types Winter may compile into strings, which for the sake of
    brevity we'll refer here as 'coercible types' are:
    - numbers
    - booleans
    - nil
    - paths
    - lists/maps known at compile-time that have elements only
      of coercible types
    - [any declared types](#declared-types) that have elements
      of coercible types
    - [any enums](#enum-syntax) that have elements of coercible types

## Variable bindings

### Let

`let` works similarly as in the `let ... in ...` from Nix, by taking
a list of variable bindings and injecting them into some 'body'
expression, which in the example below is an empty map.

```clojure
(let
  [x "value"
   y 1.2]
 {})
```

### Locals

`locals` is similar to a `let` call, but references to its bindings
can be done within the same scope, without needing to wrap all
consuming expressions inside the `let` call itself:

```clojure
(locals
  x "value"
  y "value")
{}
```

#### Scoping

Under the hood, `locals` will be compiled down to a `let` call in Nix
based on the scope:

- bindings from a `(locals)` call must precede all of its consumers
- consumers must be in the same scope, or a nested scope as the `(locals)`
  call
- a `let` will be generated, based on these rules, to accomodate the
  outer-most consumer.

##### Examples

```clojure
(locals
  special-num 1)
{favorite-num special-num}
```

Becomes:

```nix
let
  special-num = 1;
in
  {favorite-num = special-num;}
```

```clojure
(fn {config pkgs ...}
  (locals
    custom-data config.my-namespace.some-data)
  {custom-option.enable config.networking.enable
   custom-option.data custom-data
   environment.systemPackages [pkgs.btop]})
```

Becomes:

```nix
{config, pkgs, ...}: let
  custom-data = config.my-namespace.some-data;
in {
  custom-option.enable = config.networking.enable;
  custom-option.data = custom-data;
  environment.systemPackages = [pkgs.btop];
}
```

```clojure
(fn mk-module [module-name module-opts module-cfg]
  (locals
    module-extra-opts (// module-opts {enable (make-enable-option true)}))
  (fn {config ...}
    (locals
      module-extra-cfg (// module-cfg {blabla 2}))
    {options module-extra-opts
     config module-extra-cfg}))
```

Becomes:

```nix
module-name: module-opts: module-cfg: # mk-module
  {config, ...}: let
  # let only generated here, due to only consumers being here
  module-extra-opts = module-opts // {enable = make-enable-option true;};
  module-extra-cfg = module-cfg // {blabla = 2;};
  in {
    options = module-extra-opts;
    config = module-extra-cfg;
  }
```

### Inherit

Inherit has some differences from Nix's `inherit`, mainly:

- As with `locals` may be used outside of a `let` binding
- May be used within a list - so it replaces the need for the Nix's
  infamous `with` semantics.

Thus, inherit will differ in behavior depending on its placement:

- when inside a `map` - will resolve to an `inherit` inside that
  map/attrset
- when inside a `list` - will resolve to `with` blocks
- if used in a `let` - will resolve to an `inherit` inside that `let`
  block
- if used elsewhere - will resolve to a `locals` block with that
  inherit, thus resolving to a `let` with that inheritance

The syntax for inherit is inverted from Nix's counterpart in regards
to usage of parentheses to denote what is being inherited or being
inherited from:

- the namespace we're inherting a value from is *not* wrapped in
  parentheses, but its values being inherited are
- to just inherit like you would with `inherit abc;` - you also won't
  use parentheses
- Winter determines whether to inherit a namespace's objects or the
  namespace itself: to differ between what would be a
  `inherit (abc) xyz` and an `inherit abc xyz`
  - is the usage of parentheses as seen below
  - *(using parentheses will imply the items within the form wrapped in
    parentheses will be inherited from the namespace identified by the
    preceding form)*

```clojure
; on its own:
(inherit
  lib (getExe)
  pkgs (steam)
  mylib ; will inherit mylib itself
  )
; within a locals/let:
(locals
  my-var 1
  my-str "bla"
  (inherit stuff (my-bool)))
(let
  [my-var 1
   my-str "bla"
   (inherit stuff (my-bool))]
 {cool-str my-str
  cool-var my-var
  cool-bool my-bool})

; within a list:
{home.packages [(inherit pkgs (btop librewolf htop emacs))]}

; within a map:
{some-key some-val
 another-key another-val
 (inherit external-stuff (yet-another-key and-yet-another-more-key))}
```

## Functions

Nix lambdas/functions follow the form below:

```clojure
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

```clojure
(fn sum-values@num [x@num y@num]
  (+ x y))
```

## Comments

### Regular Comments

Characters in a line that are preceded by `;` are considered comments,
unless the semi-colon is inside a string.

```clojure
; this is a comment
"; this is not a comment"
```

### Documentation Comments

>[!WARNING]
>This section of the specification is work-in-progress.

Documentation comments, or simply doc-comments, are a special comment
type will be compiled-down into an appropriately-formatted
documentation comment depending on the format of choice. In Winter, it
will use a singular structure and validation, and its compiler will
compile such comments into a chosen documentation standard, with the
default being [nixdoc](https://github.com/nix-community/nixdoc).

Type descriptions are notably excluded from the syntax for these
comments because the Winter compiler will auto-resolve them from given
type annotations to the code.

The syntax for doc-comments is as follows:

```clojure
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

```clojure
(fn add@num [x@num y@num]
  (+ x y))
```

The additional type `any` exists for denoting *any* type.

#### Compound

##### Lists

*(`list@T` where T is some type)*

```clojure
(fn average@num [numbers@list@num])
```

##### Maps

*(`map@(K V)` where K and V are types for the keys and values of the
map respectively)*

```clojure
(fn tallest_person@str [people_map@map@(str num)])
```

#### Functions

Functions type annotations take the form of `fn@(X Y)` where `X` is
a list of types for every positional parameter of the function, and `Y`
is the return type.

```clojure
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

```clojure
; deletes either a file or a list of files
(fn delete_file@nil [files@(| str list@str)])
```

#### Intersections (Combined Types)

Intersections combine a new type out of two or more types, by
combining their fields. Notably this will only work with *compound*
types that are *declared* (see below in [Declared
Types](#declared-types) for more info on the topic)

The syntax is the same as unions, but using the `&` syntax.

```clojure
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
akin to Typescript:

- The Winter compiler will turn instantiated types into plain maps.

#### `type` syntax

```clojure
(type Person
  name@str
  age@num)
```

#### `enum` syntax

Maps identifiers to typed values

```clojure
(enum FileType@str
  PNG "png"
  JPEG "jpeg")

(enum FileSizes&num
  KILO 1024
  MEGA (* 1024 1024)
  GIGA (* 1024 1024 1024))
```

#### Generics

>[!WARNING]
>This section of the specification is work-in-progress.

```clojure
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

```clojure
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

```clojure
(namespace common)
(macro module! [...]
  ...)
```

```clojure
(requires
  common (module!)) ; like the `inherit` syntax
(namespace units.octodns-sync)

(module!
  "unit.octodns"
  {enable (toggle "Enable OctoDNS" false)}
  (o.when opts.enable {systemd.services.octodns-sync {}}))
```
