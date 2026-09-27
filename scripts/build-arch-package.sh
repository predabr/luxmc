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

BIN_SRC="${ROOT_DIR}/src-tauri/target/release/luxmc"
if [ ! -f "${BIN_SRC}" ]; then
    echo "Release binary not found at ${BIN_SRC}. Checking debug..."
    if [ -f "${ROOT_DIR}/src-tauri/target/debug/luxmc" ]; then
        BIN_SRC="${ROOT_DIR}/src-tauri/target/debug/luxmc"
    fi
fi

if [ -f "${BIN_SRC}" ]; then
    cp "${BIN_SRC}" "${PKG_DIR}/usr/bin/luxmc"
    chmod 755 "${PKG_DIR}/usr/bin/luxmc"
else
    echo "Warning: binary not found yet, will generate package skeleton."
    touch "${PKG_DIR}/usr/bin/luxmc"
    chmod 755 "${PKG_DIR}/usr/bin/luxmc"
fi

cp "${ROOT_DIR}/packaging/arch/luxmc.desktop" "${PKG_DIR}/usr/share/applications/luxmc.desktop"
cp "${ROOT_DIR}/packaging/arch/luxmc.desktop" "${PKG_DIR}/usr/share/applications/io.github.luxmc.Luxmc.desktop"
cp "${ROOT_DIR}/src-tauri/icons/icon.png" "${PKG_DIR}/usr/share/icons/hicolor/512x512/apps/luxmc.png"
cp "${ROOT_DIR}/src-tauri/icons/128x128.png" "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps/luxmc.png"

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

echo "Arch package generated at: ${OUT_PKG}"
