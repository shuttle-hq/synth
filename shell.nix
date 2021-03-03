{
  pkgs ? import ./nix/nixpkgs.nix {}
}:
with pkgs; mkShell {
  name = "synth-workbench";

  buildInputs = with pkgs.synthPackages; [
    rustToolchain.rust
  ] ++ synth.unwrapped.buildInputs;

  shellHook = with synth; ''
  export NIX_PYTHONPATH=${pythonEnv}/lib/python${pythonEnv.pythonVersion}/site-packages:$NIX_PYTHONPATH
  '';
}
