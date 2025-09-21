{
  description = "translate-with-bert — single-crate Nix flake (no Cargo workspace)";

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

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
        ];

        rustToolchain = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
        ];
      in rec {
        # ---------------- Package (build the single crate at repo root) ----------------
        packages.default = naerskLib.buildPackage {
          # Set a friendly pname; version comes from Cargo.toml automatically.
          pname = "translate-with-bert";
          src = ./.;

          nativeBuildInputs = nativeBuildInputs;

          # Enable tests for the single crate (disable if they require unavailable backends).
          doCheck = true;

          # If you use features, uncomment and adjust:
          # cargoBuildOptions = [ "--features" "foo,bar" ];
          # cargoTestOptions  = [ "--all-features" ];

          # If the crate builds a binary, set its name here so `nix run` can find it.
          # CHANGE THIS if your binary has a different name.
          meta.mainProgram = "translator-cli";
        };

        # ---------------- Apps (nix run) ----------------
        # Uses meta.mainProgram from the derivation.
        apps.default = {
          type = "app";
          program = pkgs.lib.getExe packages.default;
        };

        # ---------------- Dev Shell ----------------
        devShells.default = pkgs.mkShell {
          name = "translate-with-bert-devshell";

          nativeBuildInputs = nativeBuildInputs ++ rustToolchain;

          # Add runtime libs here later (CUDA/ROCm/Torch/etc.) if needed.
          buildInputs = [];

          RUST_BACKTRACE = 1;
          RUST_LOG = "info";
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

          shellHook = ''
            echo "[devshell] translate-with-bert (single crate)"
            echo "  • cargo build         # build"
            echo "  • cargo test          # run tests"
            echo "  • nix build           # build with Nix"
            echo "  • nix run             # run ${packages.default.meta.mainProgram} (adjust meta.mainProgram if needed)"
          '';
        };

        # ---------------- CI-style checks ----------------
        checks = {
          build = naerskLib.buildPackage {
            pname = "twb-build-check";
            src = ./.;
            doCheck = false;
            nativeBuildInputs = nativeBuildInputs;
          };

          test = naerskLib.buildPackage {
            pname = "twb-test-check";
            src = ./.;
            doCheck = true;
            nativeBuildInputs = nativeBuildInputs;
          };
        };

        # Optional formatter (handy for `nix fmt`)
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
