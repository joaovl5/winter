# Winter Intermediate Representation (IR)

Some things are their own IR/AST node, otherwise a `#` prefix is
introduced to avoid confusion and name-collisions.

## Operators

Operators syntax are their own IR.

## Data Types

### Scalar Data Types

- `(#num 1.2)`
- `(#bool true)`
- `(#nil)`
- `(#str "Some value")`
  - interpolated: `(#str "1+1 is " (+ 1 1))`
- `(#path "Some value")` - interpolated like `#str`

### Compound

- `(#list 1 2 3 4)`
- `(#map ("this-is-one" 1) (SomeEnumIHave.SomeVal 2))`

## Bindings

- `(let [<bind[]>] <body>)`
- `(locals <bind[]>)`
- `(#bind <name> <type> <expr>)`
  - `<type>` may be `nil` to avoid annotating it
- `(inherit <inherit-bind[]>)`
- `(#inherit-bind <from> <get...>)` - `(inherit-binding pkgs btop emacs)`
  - if `<from>` is `nil`, will compile to `inherit a b c` syntax,
    otherwise will compile as `inherit (from-value) a b c`
  - `get` is contains tuples of form `(<item> <type>)` and `type` can
    be `nil`

- Examples:
  - `(inherit globals (tld@str username@str untyped-thing))` becomes:

    ```clojure
    (inherit
      (#inherit-bind globals
        (tld str)
        (username str)
        (untyped-thing nil)))
    ```

  - `(inherit lib)` becomes:

    ```clojure
    (inherit
      (#inherit-bind nil (lib nil)))
    ```

## Functions

- `(#lambda x y)` - x is one arg, y is result
- `(#arg some-arg)`
- `(#ellipsis)` for `...` in map args
- Examples:
  - `(fn [x y] (+ x y))` becomes `(#lambda x (#lambda y (+ x y)))`
- Potential other syntaxes for baking a function can arise from this:
  - `#(x y z w)` - akin to `x: y: z: w`

## Doc Comments

*As in the spec, this is not well-defined yet.*

```clojure
(#doc-comment
  <desc>
  <samples>
  <args>)
```

Thus:

```clojure
;=
   desc: This function adds two numbers
   sample:
     (add 4 5) 9
   args:
     a The first number
     b The second number
 ;
```

Becomes:

```clojure
(#doc-comment
  "This function adds two numbers"
  ((add 4 5) 9)
  ((a "The first number")
   (b "The second number")))
```

## Types

### `<type>` values (type annotations)

- `nil`
- `str`
- `bool`
- `num`
- `list`
- `map`

#### Composed type annotations

IR forms have the `#t` prefix to denote they're type-specific.

- Unions: `(#t-union nil str)` is `@(| nil str)`
- Generics:
  - Generics have not been finalized in the spec yet, so they're not
    finalized here either!
  - `(#t-generic name)`

### Declared Types

```clojure
(type Identity
  id@num
  name@str)
```

Becomes:

```clojure
(type Identity
  (id num)
  (name str))
```

### Enums

```clojure
(enum FileType@str
  PNG "png"
  JPEG "jpeg")
```

```clojure
(enum (FileType str)
  (PNG "png")
  (JPEG "jpeg"))
```
