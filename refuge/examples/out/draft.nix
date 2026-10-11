let
  my_vale = 41;
  my_map = {
    gaba = 10;
    taga = 20;
  };
  has_gaba = my_map ? "${"gaba"}";
  tiga_or_taga = my_map."${"tiga"}" or "${(my_map)."${"taga"}"}";
in [
  (let
    a = 5;
    b = 6;
    x = "Bla bla {} ";
    y = "{{hey}}";
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
