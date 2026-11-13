{pkgs, ...}: let
  # nix's `builtins.fromTOML` breaks with TOML 1.1 so...
  # https://github.com/NixOS/nix/issues/15129
  fromTOML = path:
  # sh
    ''
      tombi format --no-cache --offline - < ${path} | yj -tj > $out
    ''
    |> pkgs.runCommandLocal "toml-to-json" {
      nativeBuildInputs = [pkgs.tombi pkgs.yj];
    }
    |> builtins.readFile
    |> builtins.fromJSON;

  workspace-cfg = fromTOML ../../Cargo.toml;
  inherit (workspace-cfg.workspace.metadata) project-name;
  src =
    pkgs.lib.fileset.gitTracked ../..
    |> (x: {
      root = ../..;
      fileset = x;
    })
    |> pkgs.lib.fileset.toSource;

  # any runtime libs needed can be put here
  runtime-libs = [];
in rec {
  default = pkgs.rustPlatform.buildRustPackage {
    pname = project-name;
    meta.mainProgram = project-name;
    meta.description = workspace-cfg.workspace.package.description;
    version = workspace-cfg.workspace.package.version;
    inherit src;
    cargoLock.lockFile = ../../Cargo.lock;

    useNextest = true;
    cargoTestFlags = ["--workspace" "--no-tests" "pass"];

    nativeBuildInputs = with pkgs; [
      clang
      mold
      makeWrapper
      pkg-config
    ];
    postFixup = ''
      wrapProgram $out/bin/${project-name} \
        --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtime-libs}
    '';
  };
  ${project-name} = default;

  rust-toolchain =
    pkgs.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;
}
