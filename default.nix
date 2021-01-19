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
, release ? false
, logLevel ? "debug"
, backtrace ? "1"
, synthSrc ? null
}:
let
  version = "0.2.1";
  gitignoreSource = filter: src: nix-gitignore.gitignoreSource filter src;
  synthdUnwrapped = naersk.buildPackage {
    name = "synth-daemon-unwrapped";
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
    ];
  };
in stdenv.mkDerivation {
  name = "synth-daemon";
  inherit version;

  src = synthdUnwrapped;

  buildInputs = [
    makeWrapper
    synthdUnwrapped
    python
  ];

  installPhase = ''
  mkdir -p $out/bin
  makeWrapper "$src/bin/synth" "$out/bin/synthd" \
              --prefix PATH ":" "${python}/bin" \
              --set RUST_BACKTRACE ${backtrace} \
              --set RUST_LOG ${logLevel}
  '';
}
