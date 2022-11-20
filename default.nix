{
  naersk
, nix-gitignore
, makeWrapper
, stdenv
, pkgconfig
, sqlite
, ncurses6
, libiconv
, darwin
, synthSrc ? null
, release ? true
}:
let
  version = "0.6.8";
  darwinBuildInputs =
    stdenv.lib.optionals stdenv.hostPlatform.isDarwin (with darwin.apple_sdk.frameworks; [
      libiconv
      IOKit
      Security
      AppKit
    ]);
  gitignoreSource = filter: src: nix-gitignore.gitignoreSource filter src;
  synth = naersk.buildPackage {
    name = "synth${suffix}";
    inherit version;

    src = if synthSrc == null then ./. else synthSrc;

    preferLocalBuild = true;

    doCheck = true;

    inherit release;

    buildInputs = [
      makeWrapper
      pkgconfig
      ncurses6.dev
      sqlite.dev
    ] ++ darwinBuildInputs;
  };
  suffix = if release then "" else "-debug";
in synth
