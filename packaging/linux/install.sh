#!/bin/sh
# Installs Penguin Viewer for you (no sudo needed). Either run it from an unpacked release:
#   sh install.sh
# or straight from the internet:
#   curl -fsSL https://github.com/Abdullahalmutairi97/Penguin-Viewer/releases/latest/download/install.sh | sh
# To remove it again:
#   sh install.sh --uninstall
set -eu

REPO=Abdullahalmutairi97/Penguin-Viewer
DATA="${XDG_DATA_HOME:-$HOME/.local/share}"
APP="$DATA/penguin-viewer-app"
BIN="$HOME/.local/bin"
APPS="$DATA/applications"
ICONS="$DATA/icons/hicolor/256x256/apps"

if [ "${1:-}" = "--uninstall" ]; then
    rm -rf "$APP"
    rm -f "$BIN/penguin-viewer" "$APPS/penguin-viewer.desktop" "$ICONS/penguin-viewer.png"
    command -v update-desktop-database >/dev/null && update-desktop-database -q "$APPS" 2>/dev/null || true
    echo "Penguin Viewer is removed."
    exit 0
fi

if [ "$(uname -m)" != x86_64 ]; then
    echo "Sorry: Penguin Viewer is only built for 64-bit Intel/AMD computers (x86_64) for now."
    exit 1
fi

# Use the files next to this script when it came in a release download; fetch them otherwise.
here=$(cd "$(dirname "$0")" 2>/dev/null && pwd || true)
if [ -n "$here" ] && [ -x "$here/penguin-viewer" ] && [ -f "$here/icon.png" ]; then
    src=$here
else
    command -v curl >/dev/null || { echo "Please install curl first."; exit 1; }
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    echo "Downloading Penguin Viewer..."
    curl -fL# "https://github.com/$REPO/releases/latest/download/penguin-viewer-linux-x86_64.tar.gz" | tar -xz -C "$tmp"
    src="$tmp/penguin-viewer"
fi

rm -rf "$APP"
mkdir -p "$APP" "$BIN" "$APPS" "$ICONS"
cp -a "$src/." "$APP/"
ln -sf "$APP/penguin-viewer" "$BIN/penguin-viewer"
cp "$APP/icon.png" "$ICONS/penguin-viewer.png"
cat > "$APPS/penguin-viewer.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Penguin Viewer
Comment=Simple image and video viewer
Exec=$APP/penguin-viewer %f
Icon=penguin-viewer
Terminal=false
Categories=Graphics;Viewer;AudioVideo;Video;
StartupWMClass=penguin-viewer
MimeType=image/png;image/jpeg;image/webp;image/gif;image/bmp;image/svg+xml;image/svg+xml-compressed;image/avif;image/tiff;image/x-icon;image/vnd.microsoft.icon;image/x-tga;image/qoi;image/x-portable-anymap;image/x-portable-bitmap;image/x-portable-graymap;image/x-portable-pixmap;image/vnd.radiance;image/x-exr;image/x-dds;video/mp4;video/x-matroska;video/webm;video/quicktime;video/x-msvideo;video/x-m4v;video/mpeg;video/ogg;video/x-ms-wmv;video/x-flv;video/3gpp;
DESKTOP
command -v update-desktop-database >/dev/null && update-desktop-database -q "$APPS" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null && gtk-update-icon-cache -q "$DATA/icons/hicolor" 2>/dev/null || true

echo "Penguin Viewer is installed. Open it from your app menu, or right-click a picture or video > Open With > Penguin Viewer."
