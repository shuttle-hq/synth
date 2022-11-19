{
  pkgs ? import ./nixpkgs.nix {}
}:
pkgs.callPackage (
  {
  , sqlite
  , synthPackages
  }:
  with synthPackages;
  let pythonEnv = python.withPackages pythonPackages;
  in mkWrappedToolchain {
    name = "wrapped-rust-toolchain";
    inherit rustToolchain;
    paths = [
      pythonEnv
    ];
    buildInputs = [
      sqlite.dev
    ];
  }
) {}
