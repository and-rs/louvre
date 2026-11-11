{
  description = "Rust Axum and Maud site development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };
          terraformRelease = {
            x86_64-linux = {
              platform = "linux_amd64";
              sha256 = "d25ce7b6902013ad905db3d2eab0be4cd905887fe88b81a6171b8d5503c31f3d";
            };
            aarch64-linux = {
              platform = "linux_arm64";
              sha256 = "8891e9dcedc9e3b8950bc6af9d4d8af1f4cfade3062f53b9dc403a89f6ce8c9c";
            };
            x86_64-darwin = {
              platform = "darwin_amd64";
              sha256 = "e2e812e783771159bf758fd4e55d6dc9bb08f63e2af2c63d212721807a02c5dc";
            };
            aarch64-darwin = {
              platform = "darwin_arm64";
              sha256 = "f210110c5698b94d803a7a63cdb0251b5455c150841478808e2bbb343f95ed68";
            };
          }.${system};
          terraformBinary = pkgs.stdenvNoCC.mkDerivation rec {
            pname = "terraform-bin";
            version = "1.15.8";
            src = pkgs.fetchurl {
              url = "https://releases.hashicorp.com/terraform/${version}/terraform_${version}_${terraformRelease.platform}.zip";
              inherit (terraformRelease) sha256;
            };
            nativeBuildInputs = [ pkgs.unzip ];
            dontUnpack = true;
            dontConfigure = true;
            dontBuild = true;
            dontStrip = true;
            installPhase = ''
              runHook preInstall
              mkdir -p "$out/bin"
              unzip "$src" terraform -d "$out/bin"
              chmod +x "$out/bin/terraform"
              runHook postInstall
            '';
            meta = {
              license = pkgs.lib.licenses.bsl11;
              platforms = systems;
              mainProgram = "terraform";
            };
          };
        in
        {
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
              terraformBinary
              awscli2
              just
              biome
              nodejs_22
              prek
              git
              postgresql_17
            ];
            shellHook = ''
              export PREK_COLOR=never
              export PREK_QUIET=1

              export REGION="us-east-1"
              export TF_VAR_target_region="$REGION"
              export AWS_DEFAULT_REGION="$REGION"
              export AWS_REGION="$REGION"
              aws configure set profile.louvre.region "$REGION" 2>/dev/null || true
              aws configure set profile.louvre.credential_process \
                "aws configure export-credentials --profile default --format process" 2>/dev/null || true
              export AWS_PROFILE="louvre"
            '';
          };
        }
      );
    };
}
