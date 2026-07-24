#!/bin/bash
set -e

# Update and install build dependencies inside Arch container
pacman -Syu --noconfirm
pacman -S --noconfirm nodejs npm rustup git webkit2gtk-4.1 gtk3 cairo pango glib2 sudo

# Configure Rust toolchain
rustup default stable

# Create non-root build user for makepkg restrictions
useradd builduser -m 2>/dev/null || true
passwd -d builduser
printf 'builduser ALL=(ALL) ALL\n' | tee -a /etc/sudoers
chown -R builduser:builduser .

# Run build process as builduser
sudo -H -u builduser bash << 'BUILDEOF'
set -e
rustup default stable
npm ci
npx tauri build --no-bundle

# Extract version from Cargo.toml
VERSION=$(grep -m1 '^version = ' src-tauri/Cargo.toml | cut -d '"' -f2 || echo "0.1.0")

# Write custom Arch PKGBUILD
cat << PKGBUILD_EOF > PKGBUILD
pkgname=bank-portal-rust
pkgver=${VERSION}
pkgrel=1
pkgdesc="Aura Trust Bank Portal - Tauri Desktop App"
arch=("x86_64")
url="https://github.com/laptop/bank-portal-rust"
license=("MIT")
depends=("webkit2gtk-4.1" "gtk3" "cairo" "pango" "glib2")

package() {
  # Install binary
  install -Dm755 "\$startdir/src-tauri/target/release/bank_portal_rust" "\$pkgdir/usr/bin/bank-portal-rust"
  # Install icon
  install -Dm644 "\$startdir/src/assets/logo.png" "\$pkgdir/usr/share/pixmaps/bank-portal-rust.png"
  
  # Install desktop entry
  mkdir -p "\$pkgdir/usr/share/applications"
  cat << DESK > "\$pkgdir/usr/share/applications/bank-portal-rust.desktop"
[Desktop Entry]
Name=Aura Trust Bank Portal
Exec=bank-portal-rust
Icon=bank-portal-rust
Terminal=false
Type=Application
Categories=Utility;Finance;
DESK
}
PKGBUILD_EOF

# Run makepkg to assemble pacman archive
makepkg -R --noconfirm
mkdir -p artifacts
cp *.pkg.tar.zst artifacts/
BUILDEOF
