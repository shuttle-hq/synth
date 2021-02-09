{
  pkgs ? import ./nix/nixpkgs.nix {}
}:
with pkgs; mkShell {
  name = "synth-workbench";

  buildInputs = with pkgs.synthPackages; [
    rustToolchain.rust
  ] ++ synth.unwrapped.buildInputs;

  shellHook = ''
  export NIX_PYTHONPATH=${synth.pythonEnv}/lib/python3.7/site-packages:$NIX_PYTHONPATH
  '';
}
