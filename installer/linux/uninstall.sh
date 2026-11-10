#!/bin/sh
# Remove only installed app files. Video files are never touched.
set -eu
prefix=${1:-"$HOME/.local"}
case "$prefix" in /*) ;; *) echo 'Choose an absolute install path.' >&2; exit 2 ;; esac
rm -f -- "$prefix/bin/anemorph" "$prefix/share/applications/anemorph.desktop" \
    "$prefix/share/icons/hicolor/256x256/apps/anemorph.png"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$prefix/share/applications" || true
fi
printf 'Removed Anemorph from %s.\n' "$prefix"
printf 'Settings and undo history are kept in ${XDG_DATA_HOME:-$HOME/.local/share}/anemorph.\n'
