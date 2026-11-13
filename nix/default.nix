let
  inputs = import ./.tack {};
  inherit
    (inputs)
    nixpkgs
    nix-vessel
    rust-overlay
    ;
  overlays = [(import rust-overlay)];
  pkgs = import nixpkgs {inherit overlays;};
in rec {
  packages = import ./packages {inherit pkgs;};

  shell = nix-vessel.mk-shell {
    inherit pkgs inputs;
    addons = ["gum" "treefmt"];
    modules = [
      {
        packages = [packages.rust-toolchain];
        inputs-from = [packages.default];
      }
      ./shell
    ];
  };
}
