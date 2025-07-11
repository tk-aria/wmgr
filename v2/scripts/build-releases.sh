#!/bin/bash
# Cross-compilation script for tsrc releases

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

VERSION=${1:-$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')}
RELEASE_DIR="target/release-packages"

echo -e "${BLUE}Building tsrc v${VERSION} for multiple targets${NC}"

# Clean and prepare
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cargo clean
mkdir -p "${RELEASE_DIR}"

# Build for each target
for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}Building for ${target}...${NC}"
    
    # Check if target is installed
    if ! rustup target list --installed | grep -q "${target}"; then
        echo -e "${YELLOW}Installing target ${target}...${NC}"
        rustup target add "${target}"
    fi
    
    # Build
    if cargo build --release --target "${target}"; then
        echo -e "${GREEN}✓ Build successful for ${target}${NC}"
        
        # Package the binary
        package_binary "${target}" "${VERSION}"
    else
        echo -e "${RED}✗ Build failed for ${target}${NC}"
        continue
    fi
done

echo -e "${GREEN}All builds completed. Packages available in ${RELEASE_DIR}${NC}"

# Generate checksums
echo -e "${YELLOW}Generating checksums...${NC}"
cd "${RELEASE_DIR}"
sha256sum *.tar.gz *.zip > checksums.txt 2>/dev/null || shasum -a 256 *.tar.gz *.zip > checksums.txt
cd - > /dev/null

echo -e "${GREEN}Release build process completed!${NC}"

function package_binary() {
    local target=$1
    local version=$2
    local binary_name="tsrc"
    local package_name="tsrc-${version}-${target}"
    
    if [[ "${target}" == *"windows"* ]]; then
        binary_name="${binary_name}.exe"
        package_name="${package_name}.zip"
        
        # Create Windows package
        cd "target/${target}/release"
        zip "../../../${RELEASE_DIR}/${package_name}" "${binary_name}"
        cd - > /dev/null
    else
        package_name="${package_name}.tar.gz"
        
        # Create Unix package
        cd "target/${target}/release"
        tar czf "../../../${RELEASE_DIR}/${package_name}" "${binary_name}"
        cd - > /dev/null
    fi
    
    echo -e "${GREEN}✓ Packaged as ${package_name}${NC}"
}

# Add install script generation
function generate_install_script() {
    cat > "${RELEASE_DIR}/install.sh" << 'EOF'
#!/bin/bash
# tsrc installation script

set -e

REPO="tk-aria/wmgr"
BINARY_NAME="tsrc"
INSTALL_DIR="${HOME}/.local/bin"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "${ARCH}" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: ${ARCH}"; exit 1 ;;
esac

case "${OS}" in
    linux)
        if ldd /bin/ls >/dev/null 2>&1; then
            TARGET="x86_64-unknown-linux-gnu"
        else
            TARGET="x86_64-unknown-linux-musl"
        fi
        ARCHIVE_EXT="tar.gz"
        ;;
    darwin)
        TARGET="${ARCH}-apple-darwin"
        ARCHIVE_EXT="tar.gz"
        ;;
    *)
        echo "Unsupported OS: ${OS}"
        exit 1
        ;;
esac

# Get latest version
VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4 | sed 's/^v//')
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_NAME}-${VERSION}-${TARGET}.${ARCHIVE_EXT}"

echo "Installing ${BINARY_NAME} v${VERSION} for ${TARGET}"

# Create install directory
mkdir -p "${INSTALL_DIR}"

# Download and extract
curl -L "${DOWNLOAD_URL}" | tar xz -C "${INSTALL_DIR}"

# Make executable
chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo "${BINARY_NAME} installed to ${INSTALL_DIR}/${BINARY_NAME}"
echo "Add ${INSTALL_DIR} to your PATH if not already included"
EOF

    chmod +x "${RELEASE_DIR}/install.sh"
    echo -e "${GREEN}✓ Generated install script${NC}"
}

generate_install_script