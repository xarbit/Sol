name := 'sol-calendar'
appid := 'io.github.xarbit.SolCalendar'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(INSTALL_DIR / 'applications' / desktop)

metainfo := appid + '.metainfo.xml'
metainfo-src := 'res' / metainfo
metainfo-dst := clean(INSTALL_DIR / 'metainfo' / metainfo)

# Default recipe to display help information
default:
    @just --list

# Run the application in debug mode
run *args:
    cargo run --release -- {{args}}

# Run the application in debug mode with debug output
run-debug *args:
    cargo run -- {{args}}

# Build the application in release mode
build-release *args:
    cargo build --release {{args}}

# Build the application in debug mode
build-debug *args:
    cargo build {{args}}

# Check the project for errors
check *args:
    cargo check --all-features {{args}}

# Run tests
test *args:
    cargo test --all-features {{args}}

# Run clippy lints
clippy *args:
    cargo clippy --all-features {{args}}

# Format the code
fmt *args:
    cargo fmt --all {{args}}

# Install the application to the system
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}

# Vendor dependencies
vendor:
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml

# Clean build artifacts
clean:
    cargo clean

# Generate Flatpak cargo dependencies manifest
flatpak-deps:
    python3 scripts/flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
    @echo "✅ Generated cargo-sources.json"

# Build Flatpak locally
flatpak-build:
    bash scripts/build-flatpak.sh

# Build standalone Flatpak bundle (.flatpak file)
flatpak-bundle:
    bash scripts/build-flatpak-bundle.sh

# Run Flatpak
flatpak-run *args:
    flatpak run {{appid}} {{args}}

# Uninstall local Flatpak
flatpak-uninstall:
    flatpak uninstall --user -y {{appid}} || true
    @echo "✅ Uninstalled {{appid}}"
