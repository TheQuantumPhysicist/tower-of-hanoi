# Tower of Hanoi

A Tower of Hanoi game built with Rust and Bevy.

## Controls

| Input | Action |
|-------|--------|
| `1` / `2` / `3` | Select source peg, then destination peg (keyboard & numpad) |
| Click | Click a peg to select, click another to move |
| Drag | Drag a top disk to another peg |
| `+` / `-` | Change number of disks (2-10) |
| `S` | Auto-solve from current state |
| `R` | Reset |
| `Esc` | Cancel selection |

## Build & Run

### Desktop

```
cargo run --release
```

### Web (WASM)

Requires [Trunk](https://trunkrs.dev/):

```
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk serve
```

Opens at `http://127.0.0.1:8080`. For a production build:

```
trunk build --release
```

Output goes to `web/dist/`.
