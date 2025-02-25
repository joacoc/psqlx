# PSQLX - A PSQL Fork Focused on Extensibility

PSQLX is an open-source project that extends PSQL by enabling custom meta-commands written in Rust or C.

## Features

- **Extensibility:** Load custom dynamic libraries to introduce new meta-commands.
- **Rust & C Support:** Write meta-commands in Rust or C.
- **Seamless Integration:** Works as a drop-in replacement for PSQL.
- **Powered by `psqlx-utils`:** Provides the necessary PSQL bindings for custom libraries.

## How Does PSQLX Work?

PSQLX is a fork of PSQL with a subtle modification that enables dynamic library loading. The core PSQL functionality remains unchanged, while additional features are handled by external libraries loaded at runtime.

## Installation

### macOS

Install `psqlx` using Homebrew:

```sh
brew install psqlx
```

### Linux

For Debian-based distributions, install `psqlx` using `apt`:

```sh
sudo apt install psqlx
```

For other Linux distributions, you may need to build from source.

### Building from Source

For building from source refer to `BUILD.md` for more instruction.

### Running PSQLX

Once installed, you can use PSQLX as a drop-in replacement for PSQL:

```sh
psqlx -U myuser -d mydatabase
```

## Creating Your Own Meta-Command

Building a meta-command is simple! Follow the instructions in `DEVELOPER.md`.

## Contributing

We welcome contributions! Feel free to submit pull requests, report issues, or suggest features.

---

### Future Plans

- Plugin discovery & management.
- More language bindings (e.g., Python, Go).
