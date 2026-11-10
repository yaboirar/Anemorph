#!/bin/sh
# Per-user installation. No administrator access required.
set -eu
prefix=${1:-"$HOME/.local"}
case "$prefix" in /*) ;; *) echo 'Choose an absolute install path.' >&2; exit 2 ;; esac
case "$prefix" in *'"'*|*'`'*|*'$'*|*'\'*|*'%'*|*'
'*) echo 'Unsupported characters in the install path.' >&2; exit 2 ;; esac
source_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install -Dm755 "$source_dir/anemorph" "$prefix/bin/anemorph"
install -Dm644 "$source_dir/icon.png" "$prefix/share/icons/hicolor/256x256/apps/anemorph.png"
mkdir -p "$prefix/share/applications"
cat > "$prefix/share/applications/anemorph.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Anemorph
Comment=Rename episodes and movies in batches
Exec="$prefix/bin/anemorph" --files %F
Icon=$prefix/share/icons/hicolor/256x256/apps/anemorph.png
Terminal=false
Categories=Utility;FileTools;
MimeType=video/x-matroska;video/mp4;video/quicktime;video/x-msvideo;video/webm;video/x-m4v;video/mpeg;video/mp2t;video/ogg;video/x-ms-wmv;video/x-flv;video/3gpp;video/3gpp2;
StartupWMClass=anemorph
EOF
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$prefix/share/applications" || true
fi
printf 'Installed Anemorph in %s\n' "$prefix"
printf 'Open your app menu or run "%s/bin/anemorph".\n' "$prefix"
printf 'For a custom prefix, add its share folder to XDG_DATA_DIRS for menu discovery.\n'
