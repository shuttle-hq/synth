{
  ref ? "master"
, rev ? "HEAD"
}:
let
  synthSrc = builtins.fetchGit {
    url = "https://github.com/openquery-io/synth.git";
    inherit ref rev;
  };
in {
  synth = (import "${synthSrc}/nix/release.nix" {}).override { inherit synthSrc; };
}
