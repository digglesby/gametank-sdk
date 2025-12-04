# gtrom

GameTank ROM build tool - a unified CLI for building, running, and managing GameTank ROM projects.

## Installation

```bash
cargo install gtrom
```

Or build from source:
```bash
cargo build -p gtrom --release
```

## Quick Start

```bash
# Create a new project
gtrom init my-game
cd my-game

# Build the ROM
gtrom build

# Run in emulator
gtrom run

# Flash to cartridge
gtrom flash
```

## Commands

### `gtrom init [PATH]`

Initialize a new GameTank project.

```bash
gtrom init my-game              # Create in new directory
gtrom init .                    # Initialize in current directory
gtrom init my-game --name "My Game"  # Set custom project name
gtrom init my-game --with-audiofw-src  # Include audio firmware source
```

Options:
- `--name <NAME>` - Set the project name (defaults to directory name)
- `--with-audiofw-src` - Include audio firmware source for customization
- `--audio <FIRMWARE>` - Audio firmware to use (default: `wavetable-8v`)

### `gtrom build`

Build the ROM. Automatically handles container orchestration for the llvm-mos toolchain.

```bash
gtrom build           # Release build (default)
gtrom build --release # Explicit release build
```

The build process:
1. Assembles `.asm` files in `src/asm/` using `llvm-mc`
2. Archives them into `libasm.a`
3. Runs `cargo build` with the mos target
4. Converts the ELF output to a `.gtr` ROM file

### `gtrom run`

Build and run the ROM in the GameTank emulator (gte).

```bash
gtrom run
```

### `gtrom flash`

Build and flash the ROM to a cartridge via gtld.

```bash
gtrom flash                    # Auto-detect serial port
gtrom flash --port /dev/ttyUSB0  # Specify port
```

### `gtrom audio <PATH>`

Build audio coprocessor firmware from an ASM or Rust project.

```bash
gtrom audio sdk/audiofw-src/wavetable-8v
```

The path should contain an `audio.toml` file with at least:
```toml
name = "wavetable-8v"
```

### `gtrom convert <ELF_PATH>`

Convert an ELF binary to a `.gtr` ROM file.

```bash
gtrom convert target/mos-unknown-none/release/my-game
gtrom convert my-game.elf --output my-game.gtr
```

## Project Structure

A gtrom project has this structure:

```
my-game/
├── rom/                    # Main ROM project
│   ├── Cargo.toml
│   ├── build.rs
│   ├── .cargo/
│   │   └── config.toml     # Target and linker settings
│   └── src/
│       ├── main.rs
│       ├── boot.rs
│       ├── sdk/            # GameTank SDK modules
│       └── asm/            # Assembly files
├── audiofw/                # Compiled audio firmware
│   └── wavetable-8v.bin
└── my-game.gtr             # Built ROM (after gtrom build)
```

## Container Build Environment

gtrom uses a podman container with the llvm-mos toolchain for building. The container is automatically managed:

- Started on first build if not running
- Mounts your workspace at `/workspace`
- Uses the `rust-mos:gte` image

When running inside the container (e.g., CI), gtrom detects this and uses the tools directly.

## Requirements

- **podman** - For container orchestration (when building outside container)
- **gte** - GameTank emulator (for `gtrom run`)
- **gtld** - GameTank loader (for `gtrom flash`)

## License

MIT
