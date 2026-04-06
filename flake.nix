{
  description = "api-hub static build with Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        staticPkgs = pkgs.pkgsStatic;
      in {
        packages.api-hub-musl = staticPkgs.rustPlatform.buildRustPackage {
          pname = "api-hub";
          version = "0.1.0";
          src = self;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          cargoBuildFlags = [ "--bin" "api-hub" ];
          doCheck = false;
          RUSTFLAGS = "-C target-cpu=x86-64 -C target-feature=-aes,-avx,-avx2";
        };

        packages.ca-certificates = pkgs.cacert;
        packages.default = self.packages.${system}.api-hub-musl;
      });
}
