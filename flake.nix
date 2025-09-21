{
  description = "translate-with-bert: Nix flake using naersk + flake-utils (workspace: translator-cli + translator-core)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        naerskLib = naersk.lib.${system};
      in {
        # ------- Packages -------
        packages = rec {
          translator-cli = naerskLib.buildPackage {
            pname = "translator-cli";
            src = ./.;

            # Build only the CLI crate within the workspace.
            cargoBuildOptions = ["--package" "translator-cli"];

            # Naersk runs `cargo test` when doCheck = true.
            doCheck = false;
          };

          default = translator-cli;
        };

        # ------- Apps (for `nix run`) -------
        apps = rec {
          translator-cli = {
            type = "app";
            program = "${self.packages.${system}.translator-cli}/bin/translator-cli";
          };
          default = translator-cli;
        };

        # ------- Dev Shell -------
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            pkg-config
            cmake
            git
            python3
          ];

          buildInputs = with pkgs; [
            # openssl
            # zlib
          ];

          # Useful for some tooling that looks for Rust sources.
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };

        # ------- Checks -------
        checks = {
          build = naerskLib.buildPackage {
            src = ./.;
            cargoBuildOptions = ["--workspace"];
            doCheck = false;
          };

          tests = naerskLib.buildPackage {
            src = ./.;
            doCheck = true;
            cargoTestOptions = ["--workspace" "--all-features"];
          };
        };
      }
    );
}
