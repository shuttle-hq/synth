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
}
