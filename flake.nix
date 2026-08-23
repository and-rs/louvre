{
  description = "Rust Axum and Maud site development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              cargo-watch
              clippy
              rust-analyzer
              rustc
              rustfmt
              rustywind
              tailwindcss_4
              brotli
              just
              biome
              prek
              git
            ];
            shellHook = ''
              export PREK_COLOR=never
              export PREK_QUIET=1
            '';
          };
        });
    };
}
