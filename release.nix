{
  pkgs ? import ./nixpkgs.nix {}
, synth ? import ./.
}:
pkgs.callPackage synth {
  release = true;
  logLevel = "info";
  backtrace = "1";
}
