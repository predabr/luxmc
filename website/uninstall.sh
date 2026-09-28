#!/usr/bin/env bash
set -euo pipefail

as_root() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    else
        sudo "$@"
    fi
}

for pkg in luxmc luxmc-bin luxmc-launcher; do
    if command -v pacman &>/dev/null && pacman -Q "$pkg" &>/dev/null; then
        as_root pacman -Rns --noconfirm "$pkg"
    elif command -v dpkg-query &>/dev/null && [ "$(dpkg-query -W -f='${Status}' "$pkg" 2>/dev/null || true)" = "install ok installed" ]; then
        as_root apt-get remove -y "$pkg"
    elif command -v rpm &>/dev/null && rpm -q "$pkg" &>/dev/null; then
        as_root dnf remove -y "$pkg"
    fi
done

remove_file() {
    if [ -e "$1" ] || [ -L "$1" ]; then
        if [ -w "$(dirname "$1")" ]; then
            rm -f -- "$1"
        else
            as_root rm -f -- "$1"
        fi
    fi
}

remove_file "$HOME/.local/bin/luxmc"
remove_file /usr/local/bin/luxmc
for base in "${XDG_DATA_HOME:-$HOME/.local/share}" /usr/local/share /usr/share; do
    for name in luxmc Luxmc io.github.luxmc.Luxmc luxmc-handler luxmc-debug-handler; do
        remove_file "$base/applications/$name.desktop"
        for apps in "$base"/icons/hicolor/*/apps; do
            for ext in png svg xpm; do
                remove_file "$apps/$name.$ext"
            done
        done
        for ext in png svg xpm; do
            remove_file "$base/pixmaps/$name.$ext"
        done
    done
    if [ -d "$base/applications" ] && command -v update-desktop-database &>/dev/null; then
        if [ -w "$base/applications" ]; then
            update-desktop-database "$base/applications"
        else
            as_root update-desktop-database "$base/applications"
        fi
    fi
    if [ -d "$base/icons/hicolor" ] && command -v gtk-update-icon-cache &>/dev/null; then
        if [ -w "$base/icons/hicolor" ]; then
            gtk-update-icon-cache -f -t "$base/icons/hicolor"
        else
            as_root gtk-update-icon-cache -f -t "$base/icons/hicolor"
        fi
    fi
done
printf '%s\n' 'Luxmc desinstalado. Mundos, modpacks e dados pessoais foram preservados.'
