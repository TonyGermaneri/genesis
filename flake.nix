{
  description = "Project Genesis - GPU-accelerated action RPG engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
        
        # Native dependencies for wgpu
        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
        ];
        
        # Runtime dependencies
        buildInputs = with pkgs; [
          # Vulkan
          vulkan-loader
          vulkan-headers
          vulkan-tools
          vulkan-validation-layers
          
          # X11 (Linux)
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          
          # Wayland (Linux)
          wayland
          libxkbcommon
          
          # macOS frameworks (conditional)
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Metal
          darwin.apple_sdk.frameworks.QuartzCore
          darwin.apple_sdk.frameworks.AppKit
          darwin.apple_sdk.frameworks.Security
        ];
        
        # Development tools
        devTools = with pkgs; [
          just                    # Command runner
          cargo-watch             # File watcher
          cargo-audit             # Security audit
          cargo-outdated          # Dependency checker
          cargo-llvm-cov          # Code coverage
          git                     # Version control
          renderdoc               # GPU debugger (Linux only)
        ];
        
      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          packages = [ rustToolchain ] ++ devTools;
          
          # Environment variables for GPU access
          shellHook = ''
            export RUST_LOG=info
            export WGPU_BACKEND=vulkan,metal
            
            # Vulkan ICD (Linux)
            export VK_ICD_FILENAMES=${pkgs.vulkan-loader}/share/vulkan/icd.d/nvidia_icd.json:${pkgs.vulkan-loader}/share/vulkan/icd.d/radeon_icd.x86_64.json
            
            # Validation layers
            export VK_LAYER_PATH=${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d
            
            echo "🎮 Project Genesis development environment"
            echo "   Rust: $(rustc --version)"
            echo "   Cargo: $(cargo --version)"
            echo ""
            echo "Commands:"
            echo "   just build     - Build all crates"
            echo "   just test      - Run tests"
            echo "   just validate  - Full validation loop"
            echo "   just run       - Run the engine"
          '';
          
          # For rust-analyzer
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
        
        # Build the engine package
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "genesis-engine";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit buildInputs nativeBuildInputs;
          
          # Skip tests in Nix build (run separately)
          doCheck = false;
          
          meta = with pkgs.lib; {
            description = "Project Genesis - GPU-accelerated action RPG engine";
            license = with licenses; [ mit asl20 ];
            platforms = platforms.all;
          };
        };
        
        # CI check
        checks.default = self.packages.${system}.default;
      });
}
