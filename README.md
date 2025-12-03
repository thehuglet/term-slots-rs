# term-slots-rs

> Spin the slots, pick the cards and play the poker hands!

[![CI Badge](https://img.shields.io/github/actions/workflow/status/thehuglet/term-slots-rs/release.yml)](https://github.com/thehuglet/term-slots-rs/actions)
[![Repo Badge](https://img.shields.io/badge/repo-thehuglet/term--slots--rs-1370D3?style=flat-square&logo=github)](https://github.com/thehuglet/term-slots-rs)
[![Rust Badge](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)

![Release Header](https://github.com/thehuglet/term-slots-rs/blob/main/assets/release-header.png)

## Running the Game

1. Grab the latest binary from [releases](https://github.com/thehuglet/term-slots-rs/releases):
   - **Windows**: `term-slots-windows.exe`
   - **Linux**: `term-slots-linux`
2. Run it:
   - **Windows**: Run `term-slots-windows.exe` or use `.\term-slots-windows.exe` in the terminal
   - **Linux**: `chmod +x term-slots-linux && ./term-slots-linux`

## CLI Args

- `--fps <n>`: Framerate limit. `0` = uncapped. Default: `144`.

## Build from Source

```bash
git clone https://github.com/thehuglet/term-slots-rs.git
cd term-slots-rs
cargo build --release
```
