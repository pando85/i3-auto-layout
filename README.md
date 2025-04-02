# i3-auto-layout

Automatic, optimal tiling for i3wm inspired by the deprecated `i3-alternating-layouts` and bspwm. An appropriate split is set for each window based on its geometry.

Improvements over `i3-alternating-layouts`:

- single compiled binary with no dependencies (except i3 of course)
- written in Rust for maximum performance and low resource usage (~0% CPU, ~0% MEM)
- works asynchronously over IPC

## Before

![image](https://raw.githubusercontent.com/pando85/i3-auto-layout/master/assets/before.png)

## After

![image](https://raw.githubusercontent.com/pando85/i3-auto-layout/master/assets/after.png)

## Installation

### Arch Linux

```bash
yay -S i3-auto-layout
```

or the binary from the AUR:

```bash
yay -S i3-auto-layout-bin
```

### Binaries

Binaries are made available each release.

You can download a prebuilt binary from our [Releases](https://github.com/pando85/i3-auto-layout/releases).

```bash
curl -s https://api.github.com/repos/pando85/i3-auto-layout/releases/latest \
  | grep browser_download_url \
  | grep -v sha256 \
  | grep $(uname -m) \
  | grep linux \
  | cut -d '"' -f 4 \
  | xargs curl -L \
  | tar xvz
sudo mv i3-auto-layout /usr/local/bin
```

### Configuration

In your i3 config:

```conf
exec_always --no-startup-id i3-auto-layout
```

## Debug

```bash
RUST_LOG=debug i3-auto-layout
```
