#!/usr/bin/env bash
set -euo pipefail

PKGNAME="luxmc"
PKGREL="1"
ARCH="x86_64"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PKGVER="$(node -p 'require(process.argv[1]).version' "${ROOT_DIR}/src-tauri/tauri.conf.json")"
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

cd "${ROOT_DIR}"
pnpm build
pnpm tauri build --no-bundle --target x86_64-unknown-linux-gnu -- --locked
TARGET_DIR="$(cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 | node -e 'let s="";process.stdin.on("data",d=>s+=d);process.stdin.on("end",()=>console.log(JSON.parse(s).target_directory))')"
BIN_SRC="${TARGET_DIR}/x86_64-unknown-linux-gnu/release/luxmc"
test -s "$BIN_SRC"
test -x "$BIN_SRC"
file "$BIN_SRC" | grep -q 'ELF 64-bit.*x86-64'

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
    tar --owner=0 --group=0 -cf - .PKGINFO usr | zstd -c -T0 -19 - > "${OUT_PKG}.partial"
)
mv "${OUT_PKG}.partial" "${OUT_PKG}"
rm -rf "${BUILD_DIR}"

echo "Arch package generated at: ${OUT_PKG}"
