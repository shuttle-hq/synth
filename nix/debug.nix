{ pkgs ? import ./nixpkgs.nix {} }: pkgs.synth.override {
  release = false;
}
