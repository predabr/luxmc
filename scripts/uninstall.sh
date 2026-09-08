#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[info]${NC} $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC} $*"; }
error() { echo -e "${RED}[error]${NC} $*" >&2; }

confirm() {
	local prompt="$1"
	echo -en "${YELLOW}${prompt} [y/N]: ${NC}"
	read -r answer
	[[ "$answer" =~ ^[Yy]$ ]]
}

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/luxmc"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/luxmc"
CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/luxmc"

echo ""
echo "=== Luxmc Uninstaller ==="
echo ""

removed=0

# --- Uninstall via paru/pacman if installed ---
if pacman -Qi luxmc &>/dev/null; then
	if confirm "Uninstall luxmc package via paru/pacman?"; then
		if command -v paru &>/dev/null; then
			paru -Rns luxmc --noconfirm
		else
			sudo pacman -Rns luxmc --noconfirm
		fi
		info "Package uninstalled"
		((removed++))
	fi
else
	info "luxmc package not found in pacman (not installed via paru)"
fi

# --- Remove user data ---
echo ""
echo "--- User data ---"

for dir in "$DATA_DIR" "$CONFIG_DIR" "$CACHE_DIR"; do
	if [[ -d "$dir" ]]; then
		echo ""
		info "Found: $dir"
		if command -v du &>/dev/null; then
			echo "  Size: $(du -sh "$dir" 2>/dev/null | cut -f1)"
		fi
		if confirm "Remove $dir?"; then
			rm -rf "$dir"
			info "Removed $dir"
			((removed++))
		fi
	fi
done

echo ""
if [[ $removed -eq 0 ]]; then
	warn "Nothing was removed."
else
	info "Done. $removed item(s) removed."
fi
echo ""
