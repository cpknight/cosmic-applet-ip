name    := 'cosmic-applet-ip'
appid   := 'com.cpknight.CosmicAppletIp'

rootdir := ''
prefix  := '/usr'

base-dir          := absolute_path(clean(rootdir / prefix))
cargo-target-dir  := env('CARGO_TARGET_DIR', 'target')

bin-src      := cargo-target-dir / 'release' / name
bin-dst      := base-dir / 'bin' / name
desktop-dst  := base-dir / 'share' / 'applications' / appid + '.desktop'
appdata-dst  := base-dir / 'share' / 'metainfo' / appid + '.metainfo.xml'
icon-dst     := base-dir / 'share' / 'icons' / 'hicolor' / 'symbolic' / 'apps' / appid + '-symbolic.svg'

# Default recipe.
default: build-release

# Clean build artifacts.
clean:
    cargo clean

# Debug build.
build-debug *args:
    cargo build {{args}}

# Release build.
build-release *args: (build-debug '--release' args)

# Run locally (useful for hacking).
run *args:
    env RUST_BACKTRACE=1 RUST_LOG=info cargo run --release {{args}}

# Install the applet system-wide. Requires root (use `sudo just install`).
install: build-release
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 data/{{appid}}.desktop {{desktop-dst}}
    install -Dm0644 data/{{appid}}.metainfo.xml {{appdata-dst}}
    install -Dm0644 data/icons/{{appid}}-symbolic.svg {{icon-dst}}
    @echo "Installed. Add it to your panel via:"
    @echo "  Settings → Desktop → Panel → Configure panel applets → Add applet"

# Remove all installed files.
uninstall:
    rm -f {{bin-dst}}
    rm -f {{desktop-dst}}
    rm -f {{appdata-dst}}
    rm -f {{icon-dst}}
    @echo "Uninstalled."

# Pull the latest from git, rebuild, and reinstall.
update:
    git pull --ff-only
    just install

# Clippy check.
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic
