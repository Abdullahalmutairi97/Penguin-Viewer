# Penguin Viewer: the easy guide

This guide is for everyone. You only need to know how to click, copy and paste.

**What Penguin Viewer does**

- 🖼️ Opens your pictures: photos (JPEG), screenshots (PNG), WebP, GIF, AVIF, SVG, TIFF and more.
- 🎬 Plays your videos with sound: MP4, MKV, WebM, MOV, AVI and more.
- ⚡ It's small and fast, and needs nothing else installed.

---

## Part 1: Install the app

### On Windows

1. Open this page: **https://github.com/Abdullahalmutairi97/Penguin-Viewer/releases/latest**
2. Scroll down to **Assets** and click **PenguinViewer-Setup.exe**. It downloads.
3. Open the downloaded file (usually in your **Downloads** folder).
4. Windows may show a blue box saying **"Windows protected your PC"**. That's only because the app is new and not "signed". Click **More info**, then **Run anyway**.
5. Click **Next** until it says **Finish**.

Penguin Viewer is now in your **Start menu** and on your desktop.

### On Linux (Ubuntu, Mint, Fedora, …)

1. Open the **Terminal** app (on Ubuntu, press **Ctrl + Alt + T**).
2. Copy this whole line, paste it into the Terminal (**Ctrl + Shift + V**) and press **Enter**:

   ```
   curl -fsSL https://github.com/Abdullahalmutairi97/Penguin-Viewer/releases/latest/download/install.sh | sh
   ```

3. Wait until it says **"Penguin Viewer is installed."** You can close the Terminal.

> If it says `curl: command not found`, first paste `sudo apt install curl`, press Enter, type your computer password, then try step 2 again.

---

## Part 2: Open a picture or video

Pick whichever way is easiest:

- **Right-click** a picture or video, choose **Open with**, then **Penguin Viewer**.
- Or open Penguin Viewer and **drag a file** onto its window.

### Make it open your pictures every time (optional)

- **Windows:** right-click a picture > **Open with** > **Choose another app** > pick **Penguin Viewer** > click **Always**. Do the same once for a video if you like.
- **Linux (Ubuntu):** right-click a picture > **Open With…** > pick **Penguin Viewer** > turn on **Always use for this file type** > **Open**.

---

## Part 3: Looking at pictures

| To do this | Do this |
|---|---|
| Next / previous picture in the folder | **→** / **←** arrow keys (or **Space** / **Backspace**) |
| First / last picture | **Home** / **End** |
| Zoom in / out | Mouse **scroll wheel**, or **+** / **−** |
| See it at real size | **Double-click** the spot you want, or press **1** |
| Fit it back to the window | **Double-click** again, or press **0** |
| Move around a zoomed picture | Click and **drag** |
| Turn it | **R** |
| Full screen | **F** (or **F11**). **Esc** to leave. |
| Close | **Q** |

---

## Part 4: Watching videos

| To do this | Do this |
|---|---|
| Pause / play | **Space** (or click the video) |
| Jump forward / back | **→** / **←** |
| Louder / quieter | **↑** / **↓** |
| Mute | **M** |
| Full screen | **Double-click** the video, or **F**. **Esc** to leave. |
| Next / previous file | **Page Down** / **Page Up** (or **N** / **P**) |

---

## Updating to a new version

- **Windows:** download the newest **PenguinViewer-Setup.exe** from the same page and run it.
- **Linux:** run the same one line from Part 1 again.

## Removing the app

- **Windows:** **Settings > Apps**, find **Penguin Viewer**, click **Uninstall**.
- **Linux:** paste this in the Terminal and press Enter:
  `sh ~/.local/share/penguin-viewer-app/install.sh --uninstall`

---

## Something isn't working?

| What you see | What to do |
|---|---|
| "Windows protected your PC" | Click **More info**, then **Run anyway**. |
| Penguin Viewer isn't in "Open with" | Close and reopen your file window. On Windows, use **Choose another app** and scroll down. |
| "Can't open this image" | The file may be damaged, or it's a type the app doesn't read yet (for example iPhone HEIC photos). |
| "Can't play this video" | The file may be damaged or incomplete (for example, still downloading). |
| No sound | Check your computer's volume, and press **M** in case the video is muted. |
