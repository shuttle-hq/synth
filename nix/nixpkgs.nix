{
  sources ? import ./sources.nix
}:
let
  nixpkgs-mozilla-rust = import "${sources.nixpkgs-mozilla}/rust-overlay.nix";
  synthOverlay = import ./overlay.nix { inherit sources; };
in import sources.nixpkgs ({
  overlays = [
    (self: super: {
      stdenv = super.stdenv // {
        lib = self.lib;
      };
    })
    nixpkgs-mozilla-rust
    synthOverlay
  ];
})
