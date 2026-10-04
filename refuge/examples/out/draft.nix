[
  (let
    a = 5;
    b = 6;
  in
    x: y: z: let my_val = 10; in x + (y + (a + my_val)))
  ({pkgs, ...}: [(pkgs).foo (pkgs).bar (pkgs).baz])
]
