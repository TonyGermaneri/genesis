# Installation

Detailed installation instructions for all platforms.

## System Requirements

### Minimum
- **OS**: Windows 10 (1903+), macOS 12+, Linux (glibc 2.31+)
- **CPU**: 64-bit processor, 4 cores
- **RAM**: 8 GB
- **GPU**: Vulkan 1.2, Metal, or DirectX 12 capable
- **Storage**: 2 GB available space

### Recommended
- **CPU**: 8+ core processor
- **RAM**: 16 GB
- **GPU**: Dedicated GPU with 4+ GB VRAM
- **Storage**: 5 GB (to accommodate mods)

## Installation Methods

### From Binary Releases

1. Visit the [Releases page](https://github.com/tonygermaneri/genesis/releases)
2. Download the appropriate archive:
   - Linux: `genesis-vX.X.X-x86_64-unknown-linux-gnu.tar.gz`
   - macOS Intel: `genesis-vX.X.X-x86_64-apple-darwin.tar.gz`
   - macOS Apple Silicon: `genesis-vX.X.X-aarch64-apple-darwin.tar.gz`
   - Windows: `genesis-vX.X.X-x86_64-pc-windows-msvc.zip`
3. Extract the archive
4. Run the `genesis` executable

### From Source (Cargo)

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/tonygermaneri/genesis.git
cd genesis
cargo build --release

# Binary is at target/release/genesis
```

### Using Nix

```bash
# One-shot run
nix run github:tonygermaneri/genesis

# Or install
nix profile install github:tonygermaneri/genesis
```

## Platform-Specific Notes

### Linux

Install Vulkan drivers for your GPU:

```bash
# NVIDIA
sudo apt install nvidia-vulkan-icd

# AMD
sudo apt install mesa-vulkan-drivers

# Intel
sudo apt install mesa-vulkan-drivers intel-media-va-driver
```

### macOS

Genesis uses the Metal backend automatically. No additional drivers needed.

**Note for Apple Silicon**: Use the `aarch64-apple-darwin` build for best performance.

### Windows

Install the latest drivers for your GPU:
- [NVIDIA Drivers](https://www.nvidia.com/drivers)
- [AMD Drivers](https://www.amd.com/support)
- [Intel Drivers](https://www.intel.com/content/www/us/en/download-center/home.html)

## Verifying Installation

Run Genesis with the `--version` flag:

```bash
./genesis --version
# Output: genesis 0.1.0
```

## Troubleshooting

### "No suitable GPU adapter found"

Your GPU doesn't support the required graphics API. Check:
- GPU drivers are up to date
- GPU supports Vulkan 1.2 (Linux/Windows) or Metal (macOS)

### "Failed to create window"

Display server issues. On Linux, ensure you have either X11 or Wayland running.

### Performance Issues

Try these steps:
1. Update GPU drivers
2. Close other GPU-intensive applications
3. Lower in-game graphics settings
4. Use the `--low-spec` flag for reduced effects

## Next Steps

- [Quick Start](quick-start.md) - Get playing quickly
- [First Steps](first-steps.md) - Learn the basics
