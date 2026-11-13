{pkgs, ...}: {
  packages = with pkgs; [
    clang
    mold
    bacon
    rust-analyzer
    cargo-nextest

    alejandra
    rumdl
    tombi

    just
  ];
}
