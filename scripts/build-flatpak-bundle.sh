#!/bin/bash
set -e

APP_ID="io.github.xarbit.SolCalendar"
BUNDLE_FILE="sol-calendar.flatpak"

echo "📦 Building Sol Calendar standalone Flatpak bundle..."
echo

# Install runtime if needed
echo "📦 Ensuring Flatpak runtime is installed..."
flatpak remote-add --if-not-exists --user flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub org.freedesktop.Platform//25.08
flatpak install --user -y flathub org.freedesktop.Sdk//25.08
flatpak install --user -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08

echo
echo "🏗️  Building Flatpak..."
flatpak-builder --user --force-clean --repo=flatpak-repo build-dir io.github.xarbit.SolCalendar.yml

echo
echo "📦 Creating standalone bundle: $BUNDLE_FILE"
flatpak build-bundle flatpak-repo "$BUNDLE_FILE" "$APP_ID"

echo
echo "✅ Build complete!"
echo "📁 Bundle created: $BUNDLE_FILE ($(du -h "$BUNDLE_FILE" | cut -f1))"
echo
echo "To install on this or another machine:"
echo "  flatpak install --user $BUNDLE_FILE"
echo
echo "To run after installing:"
echo "  flatpak run $APP_ID"
