{
  pkgs ? import ./nixpkgs.nix {}
, release ? true
, runAsRoot ? true
}:
with pkgs.dockerTools;
let
  synth = import (if release then ./release.nix else ./debug.nix) {};
  baseImage = buildImage {
    name = "synth-base";
    tag = "latest";

    contents = with pkgs; [
      sqlite.dev
      openssl.dev
    ] ++ lib.optional (! release) [
      bashInteractive
    ];
  };
in 
buildImage {
  name = "synth";
  tag = "latest";

  fromImage = baseImage;

  contents = [
    synth
  ];

  runAsRoot = if runAsRoot then null else ''
    #!${pkgs.runtimeShell}
    ${shadowSetup}
    groupadd -r synthia
    useradd -r -m -g synthia synthia
  '';

  config = {
    Entrypoint = [ "${synth}/bin/synth" ];
  } // (pkgs.lib.optionalAttrs (! runAsRoot) {
    User = "synthia";
  });
}
