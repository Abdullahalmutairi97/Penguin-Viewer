# Penguin Viewer

A simple, fast picture and video viewer. It opens almost any image (JPEG, PNG, WebP, GIF, AVIF, SVG, TIFF, ...) and plays videos with sound (MP4, MKV, WebM, MOV, AVI, ...), with nothing else to install.

**New here? Read the [easy step-by-step guide](GUIDE.md).**

## Download

Get it from the **[latest release](https://github.com/Abdullahalmutairi97/Penguin-Viewer/releases/latest)**.

- **Windows 10/11:** run **PenguinViewer-Setup.exe**. If Windows says it "protected your PC", click **More info > Run anyway** (the app isn't code-signed yet).
- **Linux** (64-bit, glibc 2.35+: Ubuntu 22.04+, Debian 12+, Fedora, Mint 21+, Arch, ...):

  ```
  curl -fsSL https://github.com/Abdullahalmutairi97/Penguin-Viewer/releases/latest/download/install.sh | sh
  ```

  It installs for your user only (no sudo). Remove it with `sh ~/.local/share/penguin-viewer-app/install.sh --uninstall`.

## Building from source

```
cargo build --release
```

On Linux this compiles ffmpeg into the binary (needs nasm, and dav1d via pkg-config). Releases are built by `.github/workflows/release.yml` when a `v*` tag is pushed; the Windows build links a ready-made LGPL ffmpeg and ships its DLLs.
