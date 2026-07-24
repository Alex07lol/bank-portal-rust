# Aura Trust Bank Portal (Tauri + Rust)

A premium, modern desktop banking management portal built with **Tauri v2 (Rust backend)** and a glassmorphic **HTML5/CSS3/JS frontend**. It provides a secure administrative ledger for bank transactions, credit loans, and investments (Fixed Deposits), directly upgrading the legacy Python/Tkinter code.

---

## 💎 Features & Built-in Examples

The app initializes a dynamic SQLite database file (`test.db`) under `Desktop/py/bank_portal/test.db` and automatically seeds the database with the following examples:
1. **Amal Dev** (Savings, Active): Active loan of Rs. 50,000 at 12.0% interest (Rs. 35,000 remaining principal).
2. **Hariprasad K.** (Checking, Active): Fully paid-off loan of Rs. 20,000 at 10.0% interest.
3. **Vishnu Prasad** (Savings, Active): Active Fixed Deposit of Rs. 100,000 at 7.0% interest (matures in 12 months).
4. **Kavya Madhavan** (Savings, Active): Matured Fixed Deposit of Rs. 30,000 at 6.0% interest (matured 1 month ago).

---

## 🛠️ Cross-Platform Implementations

Replicating code patterns from the `aether_launcher` environment:
* **Arch Linux / Linux WebKitGTK Fix**: Automatically sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` and `WEBKIT_DISABLE_COMPOSITING_MODE=1` at runtime to prevent graphic hardware-acceleration clashes (e.g. NVIDIA/Wayland white screens).
* **Windows Vibrancy Effects**: Integrates with the `window-vibrancy` crate to natively apply Acrylic, Mica, or Legacy Blur transparent styling overlays on Windows 11/10.

---

## 🚀 Development & Compilation

### Prerequisite Dependencies:
* **Arch Linux**:
  ```bash
  sudo pacman -S rustup nodejs npm webkit2gtk-4.1 gtk3 cairo pango glib2 base-devel
  ```
* **Windows**: Install Microsoft C++ Build Tools and Node.js.

### Run in Development:
```bash
npm install
npm run tauri dev
```

### Build Binary Bundles:
```bash
npm run tauri build
```
Generates `.exe` installers for Windows and `.AppImage`/`.deb` packages for Linux.

---

## 📦 CI/CD GitHub Actions Workflows

We have integrated GitHub Actions workflows for automated package compilation:
1. **Windows NSIS Installer (`build-windows.yml`)**: Compiles x64 `.exe` installers on `windows-latest`.
2. **Arch Linux Package (`build-arch.yml`)**: Assembles `.pkg.tar.zst` pacman packages inside a Dockerized Arch Linux build container (`archlinux:base-devel`).
