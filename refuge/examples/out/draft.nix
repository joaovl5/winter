let
  my_vale = 41;
in [
  (let
    a = 5;
    b = 6;
    x = "Bla bla ${1 + 1} ";
    y = "{hey}";
  in
    x: y: z: let
      my_val = 10;
      testing_if =
        if (true == true)
        then 10
        else
          (
            if (false != false)
            then 15
            else 20
          );
    in
      x + y + a + my_val + my_vale)
  ({pkgs, ...}: [((pkgs).foo) ((pkgs).bar) ((pkgs).baz)])
]
