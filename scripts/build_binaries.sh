#!/bin/bash

set -e

declare -A target_map=(
    ["aarch64-unknown-linux-gnu"]="linux arm64"
    ["aarch64-apple-darwin"]="darwin arm64"
    ["x86_64-apple-darwin"]="darwin x64"
    ["x86_64-pc-windows-gnu"]="win32 x64"
    ["x86_64-unknown-linux-gnu"]="linux x64"
)

BINARY_NAME="bls-tools"

mkdir -p bin

build_target() {
    target=$1
    platform_arch=${target_map[$target]}
    if [[ -z "$platform_arch" ]]; then
        echo "Unknown target mapping for $target"
        return 1
    fi

    echo "Building for target: $target ($platform_arch)"

    rustup target add $target || true

    # Install necessary toolchains for cross-compilation
    case "$target" in
        *windows-gnu)
            # Install mingw for GNU Windows targets
            brew install mingw-w64 || true
            ;;
        *apple-darwin)
            # Cross-compiling to different macOS architectures
            # Use rust's built-in cross-compilation support
            ;;
        *unknown-linux-gnu)
            # Cross-compiling to Linux from macOS requires additional setup
            # Use cargo zigbuild for better cross-compilation support
            ;;
    esac

    # Build the project
    if [[ "$target" == *unknown-linux-gnu ]]; then
        # Use cargo-zigbuild for cross-compiling to Linux
        cargo zigbuild --release --target $target
    else
        cargo build --release --target $target
    fi

    # Determine the output binary path
    case "$target" in
        *windows-gnu | *windows-msvc)
            binary_path="target/$target/release/${BINARY_NAME}.exe"
            ;;
        *)
            binary_path="target/$target/release/${BINARY_NAME}"
            ;;
    esac

    # Get platform and arch from mapping
    read -r platform arch <<< "$platform_arch"

    # Create the directory in bin
    mkdir -p "bin/$platform/$arch"

    # Determine the output binary name
    binary_name="${BINARY_NAME}"
    if [[ "$platform" == "win32" ]]; then
        binary_name="${binary_name}.exe"
    fi

    # Copy the binary to the bin folder
    cp "$binary_path" "bin/$platform/$arch/$binary_name"
}

# Install cargo-zigbuild for better cross-compilation support
if ! command -v cargo-zigbuild &> /dev/null; then
    echo "Installing cargo-zigbuild for cross-compilation support..."
    cargo install cargo-zigbuild
fi

# Loop over targets and build
for target in "${!target_map[@]}"; do
    # Try to build, catch any errors
    if build_target "$target"; then
        echo "Successfully built for $target"
    else
        echo "Failed to build for $target"
    fi
done

echo "All done. Binaries are in the bin/ directory."