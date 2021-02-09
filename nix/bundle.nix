{
  pkgs ? import ./nixpkgs.nix {}
, synth ? import ./release.nix { inherit pkgs; }
}:
pkgs.synthPackages.nixBundle {
  target = synth;
  run = "/bin/synth";
}
