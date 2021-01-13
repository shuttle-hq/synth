{
  pkgs ? import ./nixpkgs.nix {}
}:
pkgs.mkShell {
  name = "synth-workbench";

  buildInputs = with pkgs; [
    pkgs.synthPackages.rustToolchain.rust
    pkgconfig
    sqlite.dev
    openssl.dev
    python
  ];
}
