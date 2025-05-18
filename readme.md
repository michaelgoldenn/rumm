> [!WARNING] 
> This project is not fully stable. Make sure to backup your mods and *especially* your UserData. Please report bugs in a github issue or through discord
---
# RUMM
## Rumble's ~~Ultimate~~ Unstable Mod Manager

A mod manager for the VR game [Rumble](https://store.steampowered.com/app/890550/RUMBLE/).

This project lets you download and manage all of your mods from Thunderstore, with an emphasis on working in the background. After adding mods to the list, the manager can automatically update them to the lastest version without needing to launch them at all.

### Features
- Browsing / Downloading from Thunderstore
- Enable / Disable mods
- Easily select mod versions, and lock mods to specific versions
- Support for Windows and Linux

### Planned Features
- Self-updating for future releases
- Auto-detecting the Rumble path
- Auto-updating mods in the background (no need to start up the manager!)
- Support for installing mods locally (not from Thunderstore)
- Detecting manually installed Thunderstore mods

### Potential possible features (not immediately planned)
- Profiles for different sets of mods
- UI customization
- Automatically installing MelonLoader

## Installation

### Pre-compiled binaries
1. Go to the project’s [Releases] page.
2. Download the archive that matches your OS and CPU architecture.
3. Unpack it and move the **`rumm`** binary somewhere in your `$PATH` (e.g., `/usr/local/bin`).

### Build from source
Make sure you have a modern Rust toolchain installed (install via [rustup](https://rustup.rs/))
```bash
# Clone the repo
git clone https://github.com/michaelgoldenn/rumm.git
cd rumm

# Compile an optimized build
cargo build --release
```
then you should find the executable in target/release

#### Nix
If you use Nix flakes, after `git cloning` as before, use `nix develop` to enter the dev shell, or `nix build` to build the program
