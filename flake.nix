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

          renderer-assets = pkgs.stdenvNoCC.mkDerivation (finalAttrs: {
            pname = "${arto.pname}-renderer-assets";
            inherit (arto) version;
            src = ./renderer;

            nativeBuildInputs = [
              pkgs.nodejs-slim
              pkgs.pnpm_9.configHook
            ];

            pnpmDeps = pkgs.pnpm_9.fetchDeps {
              inherit (finalAttrs) pname version src;
              hash = "sha256-c7xJrit853qMnaY54t32kGzVDC79NPtzdiurvJ/cmJI=";
              fetcherVersion = 2;
            };

            buildPhase = ''
              runHook preBuild
              # Override output directory for Nix build
              export VITE_OUT_DIR=$out
              pnpm run build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              # Vite outputs directly to $out when VITE_OUT_DIR is set
              runHook postInstall
            '';
          });

          commonArgs = {
            src = lib.fileset.toSource rec {
              root = ./desktop;
              fileset = lib.fileset.unions [
                (craneLib.fileset.commonCargoSources root)
              ];
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
                cp -r ${renderer-assets}/* assets/dist/
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
          inherit arto renderer-assets;
        }
      );

      apps = eachSystem (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.arto}/Applications/Arto.app/Contents/MacOS/arto";
        };
      });

      devShells = eachSystem (system: {
        default =
          let
            pkgs = nixpkgs.legacyPackages.${system};
            craneLib = crane.mkLib pkgs;
          in
          craneLib.devShell {
            inputsFrom = with self.packages.${system}; [ renderer-assets ];
            packages = [
              # Rust development tools (desktop/)
              pkgs.cargo
              pkgs.rustc
              pkgs.rustfmt
              pkgs.clippy
              pkgs.rust-analyzer

              # Dioxus desktop development
              pkgs.dioxus-cli

              # TypeScript/renderer development (renderer/)
              pkgs.nodejs-slim
              pkgs.pnpm_9

              # Build automation
              pkgs.just
            ];

            # Workaround: Nix sets DEVELOPER_DIR to its apple-sdk, which breaks `just build` dmg creation.
            # https://github.com/NixOS/nixpkgs/issues/355486
            shellHook = ''
              unset DEVELOPER_DIR
              echo "🦀 Rust development environment"
              echo "  - cargo: $(cargo --version)"
              echo "  - rustc: $(rustc --version)"
              echo "  - dioxus-cli: $(dx --version)"
              echo ""
              echo "📦 TypeScript development environment"
              echo "  - node: $(node --version)"
              echo "  - pnpm: $(pnpm --version)"
              echo ""
              echo "🔧 Build tools"
              echo "  - just: $(just --version)"
            '';
          };
      });
    };
}
