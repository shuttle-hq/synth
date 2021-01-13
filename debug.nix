{
  pkgs ? import ./nixpkgs.nix {}
, synthd ? import ./.
}:
pkgs.callPackage synthd {}
