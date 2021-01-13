{
  sources ? import ./nix/sources.nix
}:
let
  nixpkgs-mozilla-rust = import "${sources.nixpkgs-mozilla}/rust-overlay.nix";
  synthOverlay = import ./overlay.nix { inherit sources; };
in import sources.nixpkgs ({
  overlays = [ nixpkgs-mozilla-rust synthOverlay ];
})
