#!/usr/bin/env bash
# ==============================================================================
# Luxmc Launcher - Linux Complete Uninstaller
# https://luxmc-r92.pages.dev
# ==============================================================================
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${RED}${BOLD}"
echo "    __                                  "
echo "   / /   __  ___  ______ ___  _____    "
echo "  / /   / / / / |/_/ __ \`__ \/ ___/    "
echo " / /___/ /_/ />  </ / / / / / /__      "
echo "/_____/\__,_/_/|_/_/ /_/ /_/\___/      "
echo -e "${NC}"
echo -e "${CYAN}${BOLD}==> Desinstalador Completo do Luxmc Launcher${NC}\n"

if command -v pacman &>/dev/null; then
    for pkg in luxmc luxmc-bin luxmc-launcher; do
        if pacman -Qi "$pkg" &>/dev/null; then
            echo -e "${YELLOW}[*] Removendo pacote $pkg via pacman...${NC}"
            if command -v sudo &>/dev/null; then
                sudo pacman -Rns "$pkg" --noconfirm 2>/dev/null || true
            else
                pacman -Rns "$pkg" --noconfirm 2>/dev/null || true
            fi
        fi
    done
fi

BIN_TARGETS=(
    "$HOME/.local/bin/luxmc"
    "/usr/local/bin/luxmc"
)
for bin in "${BIN_TARGETS[@]}"; do
    if [ -f "$bin" ] || [ -L "$bin" ]; then
        echo -e "${YELLOW}[*] Removendo executável: $bin${NC}"
        rm -f "$bin"
    fi
done

DESKTOP_TARGETS=(
    "$HOME/.local/share/applications/luxmc.desktop"
    "$HOME/.local/share/applications/io.github.luxmc.Luxmc.desktop"
    "$HOME/.local/share/applications/luxmc-handler.desktop"
    "$HOME/.local/share/applications/luxmc-debug-handler.desktop"
)
for dt in "${DESKTOP_TARGETS[@]}"; do
    if [ -f "$dt" ]; then
        echo -e "${YELLOW}[*] Removendo atalho: $dt${NC}"
        rm -f "$dt"
    fi
done

ICON_TARGETS=(
    "$HOME/.local/share/icons/hicolor/512x512/apps/luxmc.png"
    "$HOME/.local/share/icons/hicolor/128x128/apps/luxmc.png"
    "$HOME/.local/share/icons/hicolor/scalable/apps/luxmc.png"
    "$HOME/.local/share/pixmaps/luxmc.png"
)
for ic in "${ICON_TARGETS[@]}"; do
    if [ -f "$ic" ]; then
        echo -e "${YELLOW}[*] Removendo ícone: $ic${NC}"
        rm -f "$ic"
    fi
done

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
fi

echo -e "\n${GREEN}${BOLD}[✓] Luxmc foi completamente desinstalado e todos os atalhos foram removidos com sucesso!${NC}\n"
