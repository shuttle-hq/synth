{
  sources ? import ./nix/sources.nix
}:
self: super: {
  synthPackages = {
    rustToolchain = super.rustChannelOf {
      date = "2020-11-17";
      channel = "nightly";
    };

    python = super.python37.withPackages (pp: [
      pp.faker
    ]);

    nixBundle = (import sources.nix-bundle { nixpkgs = self; }).nix-bootstrap;

    synthd = self.callPackage ./default.nix {
      release = true;
      logLevel = "info";
      backtrace = "1";
    };

    synthpy = self.callPackage ./client/synthpy/default.nix {};
  };

  python = self.synthPackages.python;

  synthd = self.synthPackages.synthd;
  synthpy = self.python.pkgs.toPythonApplication self.synthPackages.synthpy;

  naersk = self.callPackage sources.naersk {
    rustc = self.synthPackages.rustToolchain.rust;
    cargo = self.synthPackages.rustToolchain.rust;
  };
}
