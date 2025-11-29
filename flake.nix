{
  description = "Arto - the Art of Reading Markdown";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          inherit (pkgs) lib;
          craneLib = crane.mkLib pkgs;

          web-assets = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
            pname = "${arto.pname}-web-assets";
            inherit (arto) version;
            src = ./web;

            nativeBuildInputs = [
              pkgs.nodejs-slim
              pkgs.pnpm_9.configHook
            ];

            pnpmDeps = pkgs.pnpm_9.fetchDeps {
              inherit (finalAttrs) pname version src;
              hash = "sha256-lpJXpXz0sWIXlcVAOWkU+Zt9W6stxwvUnj3/QtNbjJs=";
              fetcherVersion = 2;
            };

            buildPhase = ''
              runHook preBuild
              pnpm run build:icons
              pnpm run build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p ../assets/dist
              cp -r ../assets/dist $out
              runHook postInstall
            '';
          });

          commonArgs = {
            src = lib.fileset.toSource rec {
              root = ./.;
              fileset = lib.fileset.difference (lib.fileset.unions [
                (craneLib.fileset.commonCargoSources root)
                ./assets
                ./extras
                ./Dioxus.toml
              ]) ./build.rs;
            };
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Wrapper for codesign that handles directories by recursively signing files.
          # The sigtool codesign only accepts files, but bundlers like dioxus-cli
          # call codesign on .app directories. This wrapper detects directory targets
          # and recursively signs all contained files.
          codesignWrapper = pkgs.writeShellScriptBin "codesign" ''
            args=()
            target=""

            # Parse arguments to find the target path (last non-option argument)
            for arg in "$@"; do
              if [[ "$arg" != -* ]] && [[ -e "$arg" ]]; then
                target="$arg"
              fi
              args+=("$arg")
            done

            if [[ -d "$target" ]]; then
              # For directories, recursively sign all files
              while IFS= read -r -d $'\0' f; do
                # Build new args with the file instead of the directory
                file_args=()
                for arg in "''${args[@]}"; do
                  if [[ "$arg" == "$target" ]]; then
                    file_args+=("$f")
                  else
                    file_args+=("$arg")
                  fi
                done
                ${pkgs.darwin.sigtool}/bin/codesign "''${file_args[@]}" 2>/dev/null || true
              done < <(find "$target" -type f -print0)
            else
              exec ${pkgs.darwin.sigtool}/bin/codesign "$@"
            fi
          '';

          arto = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;

              nativeBuildInputs = [
                pkgs.dioxus-cli
              ]
              ++ lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
                codesignWrapper
                pkgs.darwin.autoSignDarwinBinariesHook
                pkgs.darwin.xattr
              ];

              postPatch = ''
                mkdir -p assets/dist
                cp -r ${web-assets}/* assets/dist/
              '';

              # Use buildPhaseCargoCommand instead of cargoBuildCommand because crane's
              # additional build argument `--message-format` cannot be passed to dioxus-cli properly.
              # https://crane.dev/API.html#cranelibbuildpackage
              buildPhaseCargoCommand = ''
                dx bundle --release --platform desktop --package-types macos
              '';

              # The build output is a macOS .app bundle, and crane cannot infer the install
              # destination, so we manually install without capturing cargoBuildLog in buildPhase.
              # https://crane.dev/API.html#cranelibinstallfromcargobuildloghook
              doNotPostBuildInstallCargoBinaries = true;

              installPhaseCommand = lib.optionalString pkgs.stdenv.hostPlatform.isDarwin ''
                mkdir -p $out/Applications
                cp -r target/dx/arto/bundle/macos/bundle/macos/Arto.app $out/Applications
              '';
            }
          );
        in
        {
          default = self.packages.${system}.arto;
          inherit arto web-assets;
        }
      );
    };
}
