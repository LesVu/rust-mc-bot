{
  description = "Rust Flake shell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        { pkgs, system, ... }:
        {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc

              #Ide
              rust-analyzer
              rustfmt
              clippy
              nixfmt
              nil

              #Feature
              rustPlatform.bindgenHook
              # optional: add pkg-config support
              pkg-config
            ];

            # Certain Rust tools won't work without this
            # rust-analyzer from nixpkgs does not need this.
            # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
            # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          };
        };
    };
}
