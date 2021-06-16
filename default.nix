{
  naersk
, nix-gitignore
, makeWrapper
, wrapInEnv
, stdenv
, python
, pythonPackages
, pkgconfig
, sqlite
, openssl
, ncurses6
, libiconv
, darwin
, synthSrc ? null
, release ? true
}:
let
  version = "0.5.0";
  darwinBuildInputs =
    stdenv.lib.optionals stdenv.hostPlatform.isDarwin (with darwin.apple_sdk.frameworks; [
      libiconv
      IOKit
      Security
      AppKit
    ]);
  gitignoreSource = filter: src: nix-gitignore.gitignoreSource filter src;
  pythonEnv = python.withPackages pythonPackages;
  synthUnwrapped = naersk.buildPackage {
    name = "synth-unwrapped${suffix}";
    inherit version;

    src = if synthSrc == null then ./. else synthSrc;

    preferLocalBuild = true;

    cargoBuildOptions = (opts: opts ++ [ "--features" "api"]);

    # To help with tests on MacOS
    NIX_PYTHONPATH = "${pythonEnv}/lib/python${pythonEnv.pythonVersion}/site-packages";

    doCheck = true;

    inherit release;

    buildInputs = [
      makeWrapper
      pkgconfig
      ncurses6.dev
      sqlite.dev
      openssl.dev
      pythonEnv
    ] ++ darwinBuildInputs;
  };
  suffix = if release then "" else "-debug";
in wrapInEnv {
  inherit pythonEnv;
  drv = synthUnwrapped;
  name = "synth${suffix}-${version}";
}
