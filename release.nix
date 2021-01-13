{
  pkgs ? import ./nixpkgs.nix {}
, synthd ? import ./.
}:
pkgs.callPackage synthd {
  release = true;
  logLevel = "info";
  backtrace = "1";
}
