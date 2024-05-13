{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2311.556873.tar.gz";
    rust-overlay = {
      inputs.nixpkgs.follows = "nixpkgs";
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.1327.tar.gz";
    };
    crane = {
      url = "https://flakehub.com/f/ipetkov/crane/0.16.3.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        whisper_model = pkgs.fetchurl {
          url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin?download=true";
          hash = "sha256-xhONbVjsyDIgl+D5h8MvG+i7ChhTKj+I9zTRu/nEHl0=";
        };

        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        });

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        common_args = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          doCheck = false;

          buildInputs = [ ];
          nativeBuildInputs = with pkgs; [ pkg-config cmake alsa-lib rustPlatform.bindgenHook makeWrapper ];
        };

        deps_only = craneLib.buildDepsOnly common_args;
        crate = craneLib.buildPackage (common_args // {
          cargoArtifacts = deps_only;

          postInstall = ''
            wrapProgram $out/bin/scribe --set MODEL_PATH ${whisper_model}
          '';
        });

      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            bacon
            toolchain
          ] ++ common_args.nativeBuildInputs ++ common_args.buildInputs;
          WHISPER_DONT_GENERATE_BINDINGS = true;
          MODEL_PATH = whisper_model;
        };
        packages = {
          default = crate;
        };
      });
}
