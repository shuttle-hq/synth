{
  pkgs ? import ./nixpkgs.nix {}
}:
pkgs.mkShell {
  name = "synth-workbench";

  buildInputs = with pkgs.synthPackages; [
    rustToolchain.rust
  ] ++ synth.unwrapped.buildInputs;

  shellHook = let python = pkgs.synthPackages.python; in ''
  export NIX_PYTHONPATH=${python}/lib/python3.7/site-packages:$NIX_PYTHONPATH
  '';
}
