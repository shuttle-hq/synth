{
  pkgs ? import ./nixpkgs.nix {}
}:
pkgs.callPackage (
  {
    openssl
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
      openssl.dev
      sqlite.dev
    ];
  }
) {}
