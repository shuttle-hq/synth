{
  pkgs ? import ./nixpkgs.nix {}
}:
pkgs.callPackage ({
  openssl
, sqlite
, python
, rustToolchain
, mkWrappedToolchain
}:
mkWrappedToolchain {
  name = "wrapped-rust-toolchain";
  inherit rustToolchain;
  paths = [
    python
  ];
  buildInputs = [
    openssl.dev
    sqlite.dev
  ];
}) {}
