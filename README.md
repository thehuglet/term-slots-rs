# term-slots-rs

## Running the Game

1. Grab the latest binary from [Releases](https://github.com/thehuglet/term-slots-rs/releases)
   - **Windows**: `term-slots-windows.exe`
   - **Linux**: `term-slots-linux`
2. Run it
   - **Windows**: Run `term-slots-windows.exe` or use `.\term-slots-windows.exe` in the terminal.
   - **Linux**: `chmod +x term-slots-linux && ./term-slots-linux`

## CLI Args

- `--fps <n>`: Framerate limit. `0` = uncapped. Default: `144`.

## Build from Source

```bash
git clone https://github.com/thehuglet/term-slots-rs.git
cd term-slots-rs
cargo build --release
```
