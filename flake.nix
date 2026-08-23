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
        let pkgs = import nixpkgs {
              inherit system;
              config.allowUnfree = true;
            };
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
              terraform
              awscli2
              just
              biome
              prek
              git
            ];
            shellHook = ''
              export PREK_COLOR=never
              export PREK_QUIET=1

              export REGION="us-east-1"
              export TF_VAR_target_region="$REGION"
              export AWS_DEFAULT_REGION="$REGION"
              export AWS_REGION="$REGION"
              aws configure set profile.sanarte.region "$REGION" 2>/dev/null || true
              aws configure set profile.sanarte.credential_process \
                "aws configure export-credentials --profile default --format process" 2>/dev/null || true
              export AWS_PROFILE="sanarte"
            '';
          };
        });
    };
}
