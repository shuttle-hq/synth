{
  pkgs ? import ./nixpkgs.nix {}
, synth ? import ./.
}:
pkgs.callPackage synth {}
