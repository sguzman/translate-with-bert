{
  description = "translate-with-bert — Nix flake (no Cargo workspace): builds translator-core (lib) and translator-cli (bin) with naersk";

  inputs = {
    # Stable, 25.05 release branch
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    # Small helper for multi-system outputs
    flake-utils.url = "github:numtide/flake-utils";

    # Fast Rust builder that respects your Cargo.lock
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

        # naersk with reproducible settings
        naerskLib = naersk.lib."${system}";

        # Common native build deps (kept minimal; add more if you add backends)
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
        ];

        # Rust toolchain & common dev deps
        rustToolchain = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
        ];
      in rec {
        # ---------------- Packages (build each crate directly; no --workspace) ----------------

        # Library crate
        packages.translator-core = naerskLib.buildPackage {
          pname = "translator-core";
          root = ./translator-core; # build from the crate dir
          src = ./translator-core;

          nativeBuildInputs = nativeBuildInputs;
          doCheck = true; # run tests for the lib crate
          # If you need extra features later:
          # cargoBuildOptions = [ "--features" "foo,bar" ];
          # cargoTestOptions  = [ "--all-features" ];
        };

        # CLI crate (depends on translator-core via Cargo path dependency)
        packages.translator-cli = naerskLib.buildPackage {
          pname = "translator-cli";
          root = ./translator-cli;
          src = ./translator-cli;

          nativeBuildInputs = nativeBuildInputs;
          doCheck = true;

          # Ensure the CLI crate sees the source tree (path dep already in Cargo.toml).
          # If you later publish translator-core, you can switch to crates.io dep and drop this.
        };

        # What `nix build` produces by default
        packages.default = packages.translator-cli;

        # ---------------- Apps (nix run) ----------------
        apps.default = {
          type = "app";
          program = "${packages.translator-cli}/bin/translator-cli";
        };
        # Optional convenience aliases:
        apps.translator-cli = apps.default;

        # ---------------- Dev Shell ----------------
        devShells.default = pkgs.mkShell {
          name = "translate-with-bert-devshell";

          nativeBuildInputs = nativeBuildInputs ++ rustToolchain;

          # Put any runtime libs here if you add CUDA/ROCm/Torch later.
          buildInputs = [];

          # Helpful env
          RUST_BACKTRACE = 1;
          RUST_LOG = "info";
          # Some tools expect this for rust-analyzer
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

          shellHook = ''
            echo "[devshell] translate-with-bert"
            echo "  • cargo build -p translator-cli   # build CLI"
            echo "  • cargo test  -p translator-core  # run core tests"
            echo "  • nix run                         # run translator-cli"
          '';
        };

        # ---------------- CI-style checks ----------------
        checks = {
          # Build both crates (no workspace switches)
          build-core = naerskLib.buildPackage {
            pname = "check-build-translator-core";
            root = ./translator-core;
            src = ./translator-core;
            doCheck = false;
            nativeBuildInputs = nativeBuildInputs;
          };

          build-cli = naerskLib.buildPackage {
            pname = "check-build-translator-cli";
            root = ./translator-cli;
            src = ./translator-cli;
            doCheck = false;
            nativeBuildInputs = nativeBuildInputs;
          };

          # Run tests for each crate explicitly
          test-core = naerskLib.buildPackage {
            pname = "check-test-translator-core";
            root = ./translator-core;
            src = ./translator-core;
            doCheck = true;
            nativeBuildInputs = nativeBuildInputs;
            # cargoTestOptions = [ "--all-features" ];
          };

          test-cli = naerskLib.buildPackage {
            pname = "check-test-translator-cli";
            root = ./translator-cli;
            src = ./translator-cli;
            doCheck = true;
            nativeBuildInputs = nativeBuildInputs;
          };
        };
      }
    );
}
