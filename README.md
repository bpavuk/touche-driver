# touche driver

## Setup

Ensure `libevdev-dev` and `libudev-dev` are installed:

### Ubuntu
```bash
sudo apt-get update && sudo apt-get install libevdev-dev systemd-dev
```

### Fedora
```bash
sudo dnf install libevdev-devel
```

### Arch
```
# Install yay or another AUR helper first.

# Install libudev0 from AUR
yay -S libudev0
sudo pacman -S libevdev android-udev
```

### NixOS
Under construction!

## Run
Sometimes, the driver may not see the device because it lacks bundled `udev` rules.
The problem is, there are so-o-o many device manufacturers and lineups, and each has
different device/manufacturer IDs, so it's usually simpler to run the binary as root
(with `sudo`).

Grab the binary [here](https://github.com/bpavuk/touche-driver/releases/latest)
