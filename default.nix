{
  naersk
, nix-gitignore
, makeWrapper
, stdenv
, python
, pkgconfig
, sqlite
, openssl
, ncurses6
, libiconv
, release ? false
, logLevel ? "debug"
, backtrace ? "1"
, synthSrc ? null
}:
let
  version = "0.3.0";
  darwinBuildInputs =
    stdenv.lib.optional stdenv.hostPlatform.isDarwin libiconv;
  gitignoreSource = filter: src: nix-gitignore.gitignoreSource filter src;
  synthUnwrapped = naersk.buildPackage {
    name = "synth-unwrapped";
    inherit version;

    src = ./.;

    overrideMain = old: {
      SYNTH_SRC = if synthSrc == null then ../. else synthSrc;
    };

    preferLocalBuild = true;

    inherit release;

    buildInputs = [
      makeWrapper
      pkgconfig
      ncurses6.dev
      sqlite.dev
      openssl.dev
      python
    ] ++ darwinBuildInputs;
  };
in stdenv.mkDerivation {
  name = "synth";
  inherit version;

  src = synthUnwrapped;

  buildInputs = [
    makeWrapper
    synthUnwrapped
    python
  ];

  passthru = {
    unwrapped = synthUnwrapped;
  };

  installPhase = ''
  mkdir -p $out/bin
  makeWrapper "$src/bin/synth" "$out/bin/synth" \
              --prefix PATH ":" "${python}/bin" \
              --set RUST_BACKTRACE ${backtrace} \
              --set RUST_LOG ${logLevel}
  '';
}
