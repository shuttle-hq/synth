{
  pkgs ? import ./nixpkgs.nix {}
, synthd ? import ./release.nix { inherit pkgs; }
}:
pkgs.synthPackages.nixBundle {
  target = synthd;
  run = "/bin/synthd";
}
