{
  description = "POC Handler";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)
              customRustOverlay
            ];
          };
          customRustOverlay = final: prev: {
            fixedRust = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain).override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ ];
            };
          };

          python = pkgs.python3;
          mkdocs = python.withPackages (p: [
            p.mkdocs
            p.mkdocs-material
            p.mkdocs-material-extensions
            (p.buildPythonPackage rec {
              pname = "mkdocs-with-pdf";
              version = "0.9.3";
              src = p.fetchPypi {
                inherit pname version;
                sha256 = "sha256-vaM3XXBA0biHHaF8bXHqc2vcpsZpYI8o7WJ3EDHS4MY=";
              };
              propagatedBuildInputs = [ p.beautifulsoup4 p.mkdocs p.weasyprint p.libsass ];
              doCheck = false;
            })
            (p.buildPythonPackage rec {
              pname = "mkdocs-git-revision-date-localized-plugin";
              version = "1.1.0";
              src = p.fetchPypi {
                inherit pname version;
                sha256 = "sha256-OFF+IIQinaGhuUYOhGwnSNI4wtee/UBdG5F0qHvYHXk=";
              };
              propagatedBuildInputs = [ p.mkdocs p.GitPython ];
              doCheck = false;
            })
          ]);

          doc = pkgs.stdenv.mkDerivation {
            name = "documentation";
            src = ./.;
            buildInputs = [ mkdocs ];
            buildPhase = ''
              mkdir -p dist/documentation
              mkdocs build --site-dir dist/documentation
            '';
            installPhase = ''
              mkdir $out
              cp -R dist/* $out/
            '';
          };

          ## Coverage
          coverage-wrapper = pkgs.writeScriptBin "coverage" ''
            cargo tarpaulin --all-features --workspace --timeout 120 --out Html
          '';

          baseInputs = with pkgs; [
            ## Rust
            fixedRust
            pkg-config
            openssl
            openssl.dev
            llvm
            wllvm
            rust-bindgen
          ];

          devInputs = with pkgs; [
            ## Extra rust tools
            sqlx-cli
            cargo-audit
            cargo-edit
            cargo-outdated
            cargo-watch
            cargo-tarpaulin # test coverage
            coverage-wrapper
            ## Benchmark tool
            siege
            ## Remote cluster
            # kubectl
            # kubectx
            # awscli
            # k9s
            ## Documentation tool
            mkdocs
            ## Nix
            nixpkgs-fmt
          ];

          specificSystemInputs = with pkgs;
            (if stdenv.isDarwin then [
              darwin.apple_sdk.frameworks.Security
              darwin.apple_sdk.frameworks.SystemConfiguration
              libiconv
            ] else if stdenv.isLinux then [
              fd
            ] else [ ]);

          poc-handler-server = pkgs.rustPlatform.buildRustPackage rec  {
            name = "poc-handler";
            src = self;
            nativeBuildInputs = baseInputs ++ specificSystemInputs ++ [ pkgs.docker pkgs.docker-compose ];
            buildInputs = baseInputs ++ specificSystemInputs;
            checkPhase = ''
              echo 'Setting environment'
              export SQLX_OFFLINE=true
              echo 'Check formatting' 
              cargo fmt --all -- --check
              echo 'Check clippy'
              cargo clippy -- -D warnings
            '';
            buildPhase = ''
              echo 'Setting environment'
              export SQLX_OFFLINE=true
              echo 'Build'
              mkdir -p dist/
              cargo build --release --bin poc-handler
            '';
            installPhase = ''
              mkdir $out
              cp -R ./target/release/poc-handler $out/
            '';
            # cargoSha256 = "";
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };

          ubuntuImage = pkgs.dockerTools.pullImage {
            imageName = "ubuntu";
            finalImageTag = "23.10";
            imageDigest = "sha256:07460e809a7141b84050c6d68220c1f7e2aa58b0bc124c40a1440988bfd87e6b";
            sha256 = "sha256-NPdg0BACgs8iyYfCG6l4QOhx+YL9FOESqYTp5dayZfk=";
          };

          containerImage = pkgs.dockerTools.buildImage {
            name = "poc-handler";
            tag = "latest";
            fromImage = ubuntuImage;
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [ poc-handler-server ];
            };
            config.Cmd = [ "/poc-handler" ];
            config.Env = [ "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" ];
          };

        in
        {
          defaultPackage = poc-handler-server;
          packages = {
            doc = doc;
            server = poc-handler-server;
            docker = containerImage;
          };
          devShells.default = with pkgs; mkShell {
            name = "poc-handler-shell";
            buildInputs = baseInputs ++ devInputs ++ specificSystemInputs;
            # Name
            APP_NAME = "POC Handler";
            # Log level
            RUST_LOG = "debug";
            LOGGING_LEVELS = "debug";
            # Rust Flags
            RUSTFLAGS = "-C target_cpu=native";
          };
        }
      );
}
