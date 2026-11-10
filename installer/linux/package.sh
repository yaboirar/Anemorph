#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"
if [ "${ANEMORPH_SKIP_BUILD:-0}" != 1 ]; then
    cargo build --release --locked -p anemorph
fi
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1 | tr -d '\r')
name="Anemorph-v${version}-linux-$(uname -m)"
stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT HUP INT TERM
mkdir -p "$stage/$name" dist
cp "${CARGO_TARGET_DIR:-target}/release/anemorph" "$stage/$name/"
cp assets/icon.png LICENSE installer/linux/install.sh installer/linux/uninstall.sh "$stage/$name/"
cp installer/linux/README.md "$stage/$name/README.md"
cp -R docs/licenses "$stage/$name/licenses"
cp docs/licenses/third-party-notices.md "$stage/$name/third-party-notices.md"
tar -czf "dist/$name.tar.gz" -C "$stage" "$name"
printf '%s\n' "$root/dist/$name.tar.gz"
if command -v dpkg-deb >/dev/null 2>&1; then
    deb="$stage/debian"
    mkdir -p "$deb/DEBIAN" "$deb/usr/bin" "$deb/usr/share/applications" \
        "$deb/usr/share/icons/hicolor/256x256/apps" "$deb/usr/share/doc/anemorph"
    install -m755 "${CARGO_TARGET_DIR:-target}/release/anemorph" "$deb/usr/bin/anemorph"
    cp assets/icon.png "$deb/usr/share/icons/hicolor/256x256/apps/anemorph.png"
    cp -R docs/licenses "$deb/usr/share/doc/anemorph/"
    cp LICENSE installer/linux/README.md "$deb/usr/share/doc/anemorph/"
    cat > "$deb/usr/share/applications/anemorph.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Anemorph
Comment=Rename episodes and movies in batches
Exec=anemorph --files %F
Icon=anemorph
Terminal=false
Categories=Utility;FileTools;
MimeType=video/x-matroska;video/mp4;video/quicktime;video/x-msvideo;video/webm;video/x-m4v;video/mpeg;video/mp2t;video/ogg;video/x-ms-wmv;video/x-flv;video/3gpp;video/3gpp2;
StartupWMClass=anemorph
EOF
    cat > "$deb/DEBIAN/control" <<EOF
Package: anemorph
Version: $version
Architecture: $(dpkg --print-architecture)
Maintainer: yaboirar <yaboirar@users.noreply.github.com>
Section: utils
Priority: optional
Installed-Size: $(du -sk "$deb/usr" | cut -f1)
Depends: libc6 (>= 2.39), libgcc-s1, libxkbcommon0, libxkbcommon-x11-0, libx11-6, libxcb1, libxcursor1, libxi6, libxrandr2, libwayland-client0, libegl1, libgl1
Recommends: xdg-desktop-portal, fonts-dejavu-core, fonts-noto-cjk
Homepage: https://github.com/yaboirar/Anemorph
Description: Rename episodes and movies in batches
 Preview filename changes, choose your own naming format, and undo a batch.
 Video contents stay unchanged. No account is needed.
EOF
    dpkg-deb --root-owner-group --build "$deb" "dist/Anemorph-${version}-linux-$(dpkg --print-architecture).deb"
fi
