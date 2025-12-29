# term-slots-rs

[![Rust Badge](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![CI Badge](https://img.shields.io/github/actions/workflow/status/thehuglet/term-slots-rs/release.yml)](https://github.com/thehuglet/term-slots-rs/actions)
[![Repo Badge](https://img.shields.io/badge/repo-thehuglet/term--slots--rs-1370D3?style=flat-square&logo=github)](https://github.com/thehuglet/term-slots-rs)

> Spin the slots, pick the cards and play the poker hands!

![Release Header](https://github.com/thehuglet/term-slots-rs/blob/main/assets/release-header.png)

## Bragging section
- The game is tiny with no assets, sitting at less than 1MB for Linux and less than 0.5MB for Windows as of `v0.3.0`
- Looks charming and unlike a terminal game according to playtesters
- Still has thousands of frames to spare on the lowest end hardware I could find, even with CPU shaders in play.
- It runs on a custom low level terminal renderer I wrote, some of it's notable features:
    - Utilizes double buffering to avoid flickering
    - Uses dirty rectangles to save A LOT of frame time
    - Adds support for the alpha channel
- Features CPU frag shaders:
    - LUT Gamma correction (near zero performance impact)
    - LUT Vignette (noticeable performance impact)
    - Background noise shader (used for the green "table" parts of the UI, highest performance impact)

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
