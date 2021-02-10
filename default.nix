{
  naersk
, nix-gitignore
, makeWrapper
, stdenv
, python
, pythonPackages
, pkgconfig
, sqlite
, openssl
, ncurses6
, libiconv
, darwin
, release ? true
}:
let
  version = "0.3.0";
  darwinBuildInputs =
    stdenv.lib.optionals stdenv.hostPlatform.isDarwin (with darwin.apple_sdk.frameworks; [
      libiconv
      IOKit
      Security
    ]);
  gitignoreSource = filter: src: nix-gitignore.gitignoreSource filter src;
  pythonEnv = python.withPackages pythonPackages;
  synthUnwrapped = naersk.buildPackage {
    name = "synth-unwrapped${suffix}";
    inherit version;

    src = ./.;

    preferLocalBuild = true;

    # To help with tests on MacOS
    NIX_PYTHONPATH = "${pythonEnv}/lib/python3.7/site-packages";

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
in stdenv.mkDerivation {
  name = "synth${suffix}";
  inherit version;

  src = synthUnwrapped;

  buildInputs = [
    makeWrapper
    pythonEnv
  ];

  passthru = {
    unwrapped = synthUnwrapped;
    inherit pythonEnv;
  };

  installPhase = ''
  mkdir -p $out/bin
  makeWrapper "$src/bin/synth" "$out/bin/synth" \
              --prefix PATH ":" "${pythonEnv}/bin" \
              --prefix NIX_PYTHONPATH ":" "${pythonEnv}/lib/python3.7/site-packages"
  '';
}
