{
  pkgs ? import ./nix/nixpkgs.nix {}
}:
with pkgs; mkShell {
  name = "synth-workbench";

  buildInputs = with pkgs.synthPackages; [
    rustToolchain.rust
    cmake
    gdb
    valgrind
  ] ++ synth.buildInputs;

  packages = with pkgs.synthPackages; [
    rust-analyzer
    cargo-watch
    jq
    tree
    yarn
    beekeeper-studio
    mongodb-compass
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
