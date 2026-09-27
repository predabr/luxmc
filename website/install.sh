#!/usr/bin/env bash
# ==============================================================================
# Luxmc Launcher - Linux & Arch Linux Auto-Installer
# https://luxmc.pages.dev
# ==============================================================================

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}"
echo "    __                                  "
echo "   / /   __  ___  ______ ___  _____    "
echo "  / /   / / / / |/_/ __ \`__ \/ ___/    "
echo " / /___/ /_/ />  </ / / / / / /__      "
echo "/_____/\__,_/_/|_/_/ /_/ /_/\___/      "
echo -e "${NC}"
echo -e "${BLUE}${BOLD}==> Instalador Oficial do Luxmc Launcher (Linux & Arch Linux)${NC}\n"

# 1. Checagem de Arquitetura
ARCH="$(uname -m)"
if [ "$ARCH" != "x86_64" ]; then
    echo -e "${RED}[!] Erro: A arquitetura $ARCH ainda não possui binários oficiais pré-compilados (requer x86_64).${NC}"
    exit 1
fi

# 2. Detecção de Distribuição
DISTRO="generic"
if [ -f /etc/arch-release ]; then
    DISTRO="arch"
    echo -e "${GREEN}[✓] Sistema detectado: Arch Linux / Manjaro / EndeavourOS${NC}"
elif [ -f /etc/fedora-release ]; then
    DISTRO="fedora"
    echo -e "${GREEN}[✓] Sistema detectado: Fedora / RHEL${NC}"
elif [ -f /etc/debian_version ]; then
    DISTRO="debian"
    echo -e "${GREEN}[✓] Sistema detectado: Ubuntu / Debian / Pop!_OS / Mint${NC}"
else
    echo -e "${YELLOW}[!] Sistema operacional genérico Linux detectado.${NC}"
fi

# 3. Verificação de Dependências (WebKit2GTK 4.1 / GTK3)
check_arch_deps() {
    MISSING=()
    for pkg in webkit2gtk-4.1 gtk3 libsoup3; do
        if ! pacman -Qi "$pkg" &>/dev/null; then
            MISSING+=("$pkg")
        fi
    done
    if [ ${#MISSING[@]} -gt 0 ]; then
        echo -e "${YELLOW}[*] Dependências recomendadas ausentes: ${MISSING[*]}${NC}"
        if command -v sudo &>/dev/null; then
            echo -e "${CYAN}[*] Instalando dependências via pacman...${NC}"
            sudo pacman -S --needed --noconfirm "${MISSING[@]}" || true
        else
            echo -e "${YELLOW}[!] Por favor, execute: sudo pacman -S --needed ${MISSING[*]}${NC}"
        fi
    fi
}

check_debian_deps() {
    MISSING=()
    for pkg in libwebkit2gtk-4.1-0 libgtk-3-0 libsoup-3.0-0; do
        if ! dpkg -s "$pkg" &>/dev/null; then
            MISSING+=("$pkg")
        fi
    done
    if [ ${#MISSING[@]} -gt 0 ]; then
        echo -e "${YELLOW}[*] Dependências recomendadas ausentes: ${MISSING[*]}${NC}"
        if command -v sudo &>/dev/null; then
            echo -e "${CYAN}[*] Instalando dependências via apt...${NC}"
            sudo apt-get update -y && sudo apt-get install -y "${MISSING[@]}" || true
        fi
    fi
}

case "$DISTRO" in
    arch) check_arch_deps ;;
    debian) check_debian_deps ;;
esac

# 4. Determinação dos Caminhos de Instalação
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"

mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR" "$ICON_DIR"

APPIMAGE_DEST="$INSTALL_DIR/luxmc"

# 5. Download do Release Mais Recente do GitHub
REPO="predabr/luxmc"
echo -e "${CYAN}[*] Buscando a versão mais recente do Luxmc no GitHub...${NC}"

LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v2.0.0"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/${LATEST_TAG}/Luxmc_${LATEST_TAG#v}_amd64.AppImage"
FALLBACK_URL="https://github.com/$REPO/releases/latest/download/Luxmc_amd64.AppImage"

echo -e "${CYAN}[*] Baixando Luxmc ${LATEST_TAG}...${NC}"
if ! curl -L --progress-bar --fail "$DOWNLOAD_URL" -o "$APPIMAGE_DEST"; then
    echo -e "${YELLOW}[*] Tentando URL alternativa do release...${NC}"
    curl -L --progress-bar --fail "$FALLBACK_URL" -o "$APPIMAGE_DEST" || {
        echo -e "${RED}[!] Falha ao baixar o AppImage. Verifique sua conexão com a internet.${NC}"
        exit 1
    }
fi

chmod +x "$APPIMAGE_DEST"
echo -e "${GREEN}[✓] Binário salvo e tornado executável em: $APPIMAGE_DEST${NC}"

# 6. Download e Instalação do Ícone de Alta Resolução
ICON_URL="https://raw.githubusercontent.com/$REPO/main/src-tauri/icons/icon.png"
ICON_DEST="$ICON_DIR/luxmc.png"
curl -s -L "$ICON_URL" -o "$ICON_DEST" || true

# 7. Criação do Atalho no Menu (.desktop)
rm -f "$DESKTOP_DIR/io.github.luxmc.Luxmc.desktop" "$DESKTOP_DIR/luxmc-debug-handler.desktop"
DESKTOP_FILE="$DESKTOP_DIR/luxmc.desktop"
cat <<EOFD > "$DESKTOP_FILE"
[Desktop Entry]
Name=Luxmc
GenericName=Minecraft Launcher
Comment=Launcher de Minecraft moderno, rápido e com otimização máxima para Linux
Exec=$APPIMAGE_DEST %u
Icon=luxmc
Terminal=false
Type=Application
Categories=Game;ActionGame;AdventureGame;
StartupWMClass=luxmc
StartupNotify=true
MimeType=x-scheme-handler/luxmc;
Keywords=minecraft;launcher;luxmc;modpack;optifine;fabric;forge;neoforge;
EOFD

chmod +x "$DESKTOP_FILE"
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$DESKTOP_DIR" &>/dev/null || true
fi

# 8. Integração no PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    fi

    if [ -n "$SHELL_RC" ] && ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_RC"; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
    fi
fi

echo -e "\n${GREEN}${BOLD}================================================================${NC}"
echo -e "${GREEN}${BOLD}       Luxmc Launcher instalado com sucesso no seu Linux!      ${NC}"
echo -e "${GREEN}${BOLD}================================================================${NC}\n"
echo -e "  ${BOLD}Como iniciar:${NC}"
echo -e "  1. Pelo menu de aplicativos do seu sistema (procure por ${CYAN}Luxmc Launcher${NC})"
echo -e "  2. Pelo terminal: execute ${CYAN}luxmc${NC} ou ${CYAN}$APPIMAGE_DEST${NC}\n"
echo -e "  ${YELLOW}Dica de Performance (Arch/Fedora/Debian):${NC}"
echo -e "  Instale o pacote ${BOLD}gamemode${NC} para que o Luxmc acelere automaticamente a GPU e CPU!\n"
