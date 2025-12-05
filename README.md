What if Rust, on [GameTank](https://gametank.zone/)?

# What?

The GameTank is a retro-inspired game console running dual 6502 processors. 
It has 128x128 composite out, with up to 200 colors. It's kind of like if Pico8 were real hardware -- 
with real hardware limitations, and neat tricks!

# Why?

I like Rust, I like GameTank. What if, both?

# How?

Rust compiles via LLVM. There is [LLVM-MOS](https://github.com/llvm-mos/llvm-mos/blob/main/README.md), 
which can target the 6502. Throw all that together and create some linker scripts, we _should_ have a stew.

# GameTank SDK (Rust)

This SDK produces 2MB GameTank ROMs (`.gtr`) to be uploaded to physical cartridges, or to be played in the emulator.

Bundled with the SDK are development tools:


| tool    | description |
| ------- | ----------- |
| `gtrom` | the main build tool; initializes new projects, builds ROMs (orchestrates containers automatically), converts ELF to `.gtr`, and can run/flash directly. |
| `gte`   | the rusty gametank emulator. It's not quite as featureful as the C++ version, but it's easier to install, useful for basic debugging/testing |
| `gtld`  | used to flash `.gtr` ROMs to cartridges, and to update the flasher firmware. |
| `gtgo`  | intended to be a "one-stop-shop" TUI for development, includes a (WIP) music tracker and build tools |
    
Development is done in VSCode (sry), and there's a `.vscode/settings.json` for the linked projects for rust-analyzer.

## Requirements

- `Podman` (or Docker, either should work, but Podman is tested and easier to set up)
- `Rust`*

> It's technically possible to make GameTank games using Rust without installing Rust at all! But you probably won't have rust-analyzer support, which isn't very fun.

If you're on Windows or MacOS, you'll likely want to use Podman Desktop.

See **Windows Setup** for further details on setting up Windows.

## Installation

```bash
cargo install gametank-sdk
```

Or download an installer from [GitHub releases](https://github.com/dwbrite/gametank-sdk/releases).

## Quick Start

```bash
# Create a new project
gtrom init my-game
cd my-game

# Build the ROM (handles containers automatically)
gtrom build

# Run in emulator
gtrom run

# Flash to cartridge
gtrom flash
```

## Editor Setup

We recommend using [VS Code](https://code.visualstudio.com/) for development. New projects include a `.vscode/settings.json` for rust-analyzer.

**Recommended extensions:**
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - Rust language support

**Coming from JetBrains?** Try these to feel more at home:
- [IntelliJ IDEA Keybindings](https://marketplace.visualstudio.com/items?itemName=k--kato.intellij-idea-keybindings)
- [Darcula Theme](https://marketplace.visualstudio.com/items?itemName=rokoroku.vscode-theme-darcula)

## Windows Setup

If you're not already a Rust / WSL user, here are your options:

### Option 1: Use Prebuilt Binaries

Download the `.msi` installer or PowerShell script from [GitHub releases](https://github.com/dwbrite/gametank-sdk/releases).

You'll still need:
- [Podman Desktop](https://podman-desktop.io/) (or Docker Desktop) for building ROMs
- Rust installed for rust-analyzer in VS Code (see Option 3 for install steps)

### Option 2: WSL (Recommended for Development)

WSL gives you a Linux environment, which is generally smoother for development.

```powershell
# Install WSL
wsl --install

# After restart, install Ubuntu
wsl --install Ubuntu-24.04
```

Then inside WSL, follow the Linux installation steps.

### Option 3: Native Windows with Rust

1. Install [Visual Studio Community 2022](https://visualstudio.microsoft.com/vs/community/) with the "Desktop development with C++" workload  
   (See: [Rust Windows MSVC guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html))

2. Install Rust and Podman:
   ```powershell
   winget install Rustlang.Rustup
   winget install RedHat.Podman-Desktop
   ```

3. Then `cargo install gametank-sdk`


## Advanced: Manual Container Commands

These commands are provided for reference. Normally `gtrom build` handles all of this automatically.

```bash
# Start a persistent container
podman run -d --name gametank -v $(pwd):/workspace:z --replace dwbrite/rust-mos:gte sleep infinity

# Enter the container shell
podman exec -it -w /workspace gametank /bin/zsh

# Inside container: assemble .asm files
find . -name "*.asm" -exec bash -c 'filename=$(basename "{}" .asm); echo "Assembling $filename..."; llvm-mc --filetype=obj -triple=mos -mcpu=mosw65c02 "{}" -o "target/asm/$filename.o"' \;

# Inside container: create static library from assembled objects
llvm-ar rcs target/asm/libasm.a target/asm/*.o && rm target/asm/*.o

# Inside container: build with cargo+mos
cargo +mos build --release -Z build-std=core --target mos-unknown-none
```
