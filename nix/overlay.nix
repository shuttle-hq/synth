{
  sources ? import ./sources.nix
}:
self: super: {
  synthPackages = {
    rustToolchain = super.rustChannelOf {
      date = "2021-02-15";
      channel = "nightly";
    };

    python = super.python38;
    pythonPackages = pp: with pp; [
      faker
    ];

    nixBundle = (import sources.nix-bundle { nixpkgs = self; }).nix-bootstrap;

    wrapInEnv = {
      drv
      , name
      , pythonEnv
    }: self.stdenv.mkDerivation {
      inherit name;

      version = drv.version;

      src = drv;

      buildInputs = [
        self.makeWrapper
        pythonEnv
      ];

      passthru = {
        unwrapped = drv;
        inherit pythonEnv;
      };

      installPhase = ''
      mkdir -p $out/bin
      for bin in $src/bin/*; do
         bin_name=$(basename $bin)
         makeWrapper "$src/bin/$bin_name" "$out/bin/$bin_name" \
                     --prefix PATH ":" "${pythonEnv}/bin" \
                     --prefix NIX_PYTHONPATH ":" "${pythonEnv}/lib/python${pythonEnv.pythonVersion}/site-packages"
      done
      '';
    };

    mkWrappedToolchain = {
      name
      , buildInputs
      , paths
      , rustToolchain
    }:
      with self.lib; let
        mkFlags = prefix: suffix: xxs:
          lists.foldr (f: s: s + " ${f}") "" (map (lib: "${prefix}${lib}${suffix}") xxs);
        cFlags = mkFlags "-I" "/include" buildInputs;
        ldFlags = mkFlags "-L" "/lib" buildInputs;
        pkgConfigPath = mkFlags "" "/lib/pkgconfig" buildInputs;
        pathPrefix = lists.foldr (p: s: s + "${p}/bin:") "" paths;
      in self.symlinkJoin {
        inherit name;
        paths = [
          rustToolchain.rust
          rustToolchain.rust-src
        ];
        buildInputs = [ self.makeWrapper ];
        postBuild = ''
        for f in $out/bin/**; do
          mv $f $f.unwrapped
          makeWrapper $f.unwrapped $f \
                      --prefix PATH : "${self.pkgconfig}/bin" \
                      --prefix PATH "" "${pathPrefix}" \
                      --prefix PATH ":" "$out/bin" \
                      --prefix NIX_LDFLAGS " " "${ldFlags}" \
                      --prefix NIX_CFLAGS_COMPILE " " "${cFlags}" \
                      --suffix-each PKG_CONFIG_PATH : "${pkgConfigPath}"
        done
        '';
      };

    synth = self.callPackage ../default.nix {
      pythonPackages = self.synthPackages.pythonPackages;
    };
  };

  rustToolchain = self.synthPackages.rustToolchain;
  mkWrappedToolchain = self.synthPackages.mkWrappedToolchain;
  wrapInEnv = self.synthPackages.wrapInEnv;

  python = self.synthPackages.python;

  synth = self.synthPackages.synth;

  naersk = self.callPackage sources.naersk {
    rustc = self.synthPackages.rustToolchain.rust;
    cargo = self.synthPackages.rustToolchain.rust;
  };
}
