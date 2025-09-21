{
  description = "translate-with-bert: Nix flake using naersk + flake-utils (workspace: translator-cli + translator-core)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
        # Build the CLI from the Cargo workspace (translator-cli default binary).
        packages = rec {
          translator-cli = naerskLib.buildPackage {
            pname = "translator-cli";
            src = ./.;

            # Build only the CLI crate within the workspace.
            cargoBuildOptions = ["--package" "translator-cli"];
            # Enable running tests via `nix build .#checks.tests`
            doCheck = false;
          };

          # Expose the CLI as the default package
          default = translator-cli;
        };

        # ------- Apps (nice `nix run` UX) -------
        apps = {
          translator-cli = {
            type = "app";
            program = "${self.packages.${system}.translator-cli}/bin/translator-cli";
          };
          default = apps.translator-cli;
        };

        # ------- Dev Shell -------
        # A comfy shell for hacking, testing and building locally with cargo.
        devShells.default = pkgs.mkShell {
          # Tools commonly needed for Rust + native deps; tweak as your crates add backends.
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

          # If you add dependencies that need C toolchains or SSL, keep these handy:
          buildInputs = with pkgs; [
            # openssl
            # zlib
          ];

          # Make sure cargo sees pkg-config, etc.
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };

        # ------- Checks (CI-friendly targets) -------
        # 1) Build the whole workspace (debug) to catch compile errors early.
        checks.build = naerskLib.buildPackage {
          src = ./.;
          cargoBuildOptions = ["--workspace"];
          doCheck = false;
        };

        # 2) Run the workspace tests via `nix build .#checks.tests`
        checks.tests = naerskLib.buildPackage {
          src = ./.;
          doCheck = true;
          # naersk will run `cargo test` when doCheck = true; add options if you like:
          cargoTestOptions = ["--workspace" "--all-features"];
        };
      }
    );
}
