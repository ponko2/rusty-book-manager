{
  description = "vscode-remote-try-rust";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    inputs@{ fenix, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, system, ... }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              fenix.overlays.default
            ];
          };
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              (pkgs.fenix.fromToolchainFile {
                file = ./rust-toolchain.toml;
                sha256 = "sha256-sqSWJDUxc+zaz1nBWMAJKTAGBuGWP25GCftIOlCEAtA=";
              })
              cargo-make
              editorconfig-checker
              nodejs
              postgresql
              redis
              rust-analyzer
              yamllint
            ];
          };
          formatter = pkgs.nixfmt-tree;
        };
    };
}
