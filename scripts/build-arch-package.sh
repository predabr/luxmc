#!/usr/bin/env bash
set -e

PKGNAME="luxmc"
PKGVER="2.0.0"
PKGREL="1"
ARCH="x86_64"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/packaging/arch/pkg_build"
PKG_DIR="${BUILD_DIR}/pkg"

echo "Building Arch Linux package for ${PKGNAME} ${PKGVER}-${PKGREL}..."

rm -rf "${BUILD_DIR}"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/512x512/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/64x64/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/32x32/apps"
mkdir -p "${PKG_DIR}/usr/share/pixmaps"

BIN_SRC=""
for candidate in \
    "${ROOT_DIR}/src-tauri/target/x86_64-unknown-linux-gnu/release/luxmc" \
    "${ROOT_DIR}/target/x86_64-unknown-linux-gnu/release/luxmc" \
    "${ROOT_DIR}/src-tauri/target/release/luxmc" \
    "${ROOT_DIR}/target/release/luxmc" \
    "${ROOT_DIR}/src-tauri/target/x86_64-unknown-linux-gnu/debug/luxmc" \
    "${ROOT_DIR}/src-tauri/target/debug/luxmc"; do
    if [ -f "$candidate" ] && [ -s "$candidate" ]; then
        BIN_SRC="$candidate"
        break
    fi
done

if [ -z "${BIN_SRC}" ] || [ ! -s "${BIN_SRC}" ]; then
    echo "Error: Binary not found or empty! Cannot generate valid package."
    exit 1
fi

echo "Using binary: ${BIN_SRC} ($(du -h "${BIN_SRC}" | cut -f1))"
cp "${BIN_SRC}" "${PKG_DIR}/usr/bin/luxmc"
chmod 755 "${PKG_DIR}/usr/bin/luxmc"

cp "${ROOT_DIR}/packaging/arch/luxmc.desktop" "${PKG_DIR}/usr/share/applications/luxmc.desktop"
cp "${ROOT_DIR}/src-tauri/icons/icon.png" "${PKG_DIR}/usr/share/icons/hicolor/512x512/apps/luxmc.png"
cp "${ROOT_DIR}/src-tauri/icons/128x128.png" "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps/luxmc.png"
cp "${ROOT_DIR}/src-tauri/icons/64x64.png" "${PKG_DIR}/usr/share/icons/hicolor/64x64/apps/luxmc.png"
cp "${ROOT_DIR}/src-tauri/icons/32x32.png" "${PKG_DIR}/usr/share/icons/hicolor/32x32/apps/luxmc.png"
cp "${ROOT_DIR}/src-tauri/icons/icon.png" "${PKG_DIR}/usr/share/pixmaps/luxmc.png"

cat <<EOF > "${PKG_DIR}/.PKGINFO"
pkgname = ${PKGNAME}
pkgver = ${PKGVER}-${PKGREL}
pkgdesc = Linux-first Minecraft launcher written in Tauri 2, SvelteKit, and Rust
url = https://github.com/predabr/luxmc
builddate = $(date +%s)
packager = Luxmc <suporte@luxmc.com>
size = $(du -sb "${PKG_DIR}" | cut -f1)
arch = ${ARCH}
license = MIT
depend = webkit2gtk-4.1
depend = gtk3
depend = libsoup3
depend = openssl
depend = librsvg
depend = libayatana-appindicator
depend = hicolor-icon-theme
depend = xdg-utils
optdepend = gamemode: Feral GameMode integration for performance boost
optdepend = vulkan-radeon: RADV driver for AMD Vulkan acceleration
optdepend = vulkan-intel: ANV driver for Intel Vulkan acceleration
provides = luxmc-launcher
provides = luxmc-bin
conflict = luxmc-launcher
conflict = luxmc-bin
EOF

OUT_PKG="${ROOT_DIR}/packaging/arch/${PKGNAME}-${PKGVER}-${PKGREL}-${ARCH}.pkg.tar.zst"
(
    cd "${PKG_DIR}"
    tar -cf - .PKGINFO usr | zstd -c -T0 -19 - > "${OUT_PKG}"
)
rm -rf "${BUILD_DIR}"

echo "Arch package generated at: ${OUT_PKG}"
