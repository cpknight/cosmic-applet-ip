# cosmic-applet-ip

A tiny [COSMIC DE](https://system76.com/cosmic/) panel widget — what COSMIC
insists on calling an "applet" — that **shows the IP address of this
computer** right in your panel.

* By default it shows the IP of the interface that owns the current default
  route (the "main" IP, effectively whatever you're using to reach the
  internet).
* **Click it** to cycle through every network interface on your machine. A
  little popup shows the interface's name, whether it's connected, and its
  IPv4/IPv6 addresses.
* Whatever you cycle to is **remembered across sessions** (via
  `cosmic-config`). A **Reset to Auto** button in the popup brings you back
  to the default-route behavior.
* Poll-refreshes every 2 s, so plugging/unplugging Ethernet, joining Wi-Fi,
  starting a VPN, etc. will update the displayed IP automatically.

## Build requirements

* Rust 1.80+ (Rust edition 2024)
* A C compiler + `pkg-config` + development headers for Wayland / xkbcommon
  (the usual libcosmic build prerequisites)
* `just` is optional; there's a plain bash `install.sh` as a fallback

On Arch Linux (and derivatives) the packages you likely want are:

```bash
sudo pacman -S --needed rust just base-devel wayland libxkbcommon
```

## Install

### With `just` (recommended)

```bash
git clone https://github.com/cpknight/cosmic-applet-ip
cd cosmic-applet-ip
sudo just install
```

### Without `just`

```bash
git clone https://github.com/cpknight/cosmic-applet-ip
cd cosmic-applet-ip
./install.sh        # will sudo internally
```

Then add the applet to your panel:

> **Settings → Desktop → Panel → Configure panel applets → Add applet → "IP Address"**

## Update

```bash
cd cosmic-applet-ip
sudo just update    # git pull + rebuild + reinstall
```

…or just run `git pull && ./install.sh` if you're not using `just`.

## Uninstall

```bash
sudo just uninstall
# or
./uninstall.sh
```

## Custom install prefix

Both the justfile and the scripts honour `PREFIX` (default `/usr`). Install
into your home directory without root:

```bash
PREFIX="$HOME/.local" just install
# or
PREFIX="$HOME/.local" ./install.sh
```

## How it works

* Network data comes from `getifaddrs(3)` via the small
  [`if-addrs`](https://crates.io/crates/if-addrs) crate, plus
  `/proc/net/route` to find the default-route interface.
* The UI is pure [libcosmic](https://github.com/pop-os/libcosmic) — same
  toolkit System76 uses for the built-in applets. It registers itself with
  `cosmic-panel` via `X-CosmicApplet=true` in its `.desktop` file.
* Selection state is stored under the COSMIC config namespace
  `io.cpknight.CosmicAppletIp` (usually
  `~/.config/cosmic/io.cpknight.CosmicAppletIp/v1/`).

## License

GPL-3.0-only. Same license as System76's first-party COSMIC applets.
