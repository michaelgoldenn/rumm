> [!WARNING] 
> This project is not fully stable. Make sure to backup your mods and *especially* your UserData. Please report bugs in a github issue or through discord, and PRs are welcome.
---
# RUMM
## Rumble's ~~Ultimate~~ Unstable Mod Manager

A mod manager for the VR game [Rumble](https://store.steampowered.com/app/890550/RUMBLE/).

This project lets you download and manage all of your mods from Thunderstore. After downloading them from the list, make sure to sync the list with rumble

### Features
- Browsing / Downloading from Thunderstore
- Enable / Disable mods
- Easily select mod versions, and lock mods to specific versions
- Support for Windows and Linux

### Planned Features
- Better sorting for mod lists
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
Before installation, make sure [MelonLoader](https://melonloader.co/) is installed in Rumble so that mods will work.

### Pre-compiled binaries
1. Go to the project’s [Releases](https://github.com/michaelgoldenn/rumm/releases) page.
2. Download the file that matches your OS (`rumm.exe` for windows, `rumm` for linux)
3. Unpack the zip and run the executable (you can add it to your PATH if you want to find it easier in the future)

### Build from source
Make sure you have a modern Rust toolchain installed (install via [rustup](https://rustup.rs/))
```bash
git clone https://github.com/michaelgoldenn/rumm.git
cd rumm
cargo build --release # Can omit --release if you want a faster build
```
then you should find the executable in target/release

#### Nix
If you use Nix flakes, after `git cloning` as before, use `nix develop` to enter the dev shell, then `cargo build` should work

## Screenshots
![image](https://github.com/user-attachments/assets/426391c9-c62b-45a8-84da-d11c0f37b57b)
![image](https://github.com/user-attachments/assets/c2fb2534-c6c2-4df4-bdaa-8e9f5bdb1e5f)
