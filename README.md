# typingterm

A monkeytype-inspired typing test for the terminal, written in Rust.

## Install

```bash
git clone https://github.com/nathom/typingterm
cd typingterm
cargo install --path .
```

Or just build and run directly:

```bash
cargo run --release
```

## Usage

### Menu

Navigate with arrow keys or `hjkl`. Press `enter` to start a test, `q` to quit.

Press `/` on the language or theme rows to search.

### Test modes

- **time** - type for 15, 30, 60, or 120 seconds
- **words** - type 10, 25, 50, or 100 words
- **quote** - type a random quote
- **zen** - no timer, no word limit

### Languages

19 languages: English (200/1k/5k/10k), Spanish, French, German, Portuguese, Italian, Dutch, Swedish, plus code (Rust, Python, JavaScript, TypeScript, Go, C, C++, Java).

Code mode uses real snippets from [Rosetta Code](https://rosettacode.org). Indentation is auto-inserted.

### Themes

187 themes from monkeytype (dracula, catppuccin, nord, serika_dark, etc). Settings persist across sessions.

### Keybindings

| Key | Action |
|---|---|
| `tab` | restart test |
| `esc` | back to menu |
| `enter` | next line (code mode) |
| `ctrl+backspace` / `opt+backspace` | delete word |
| `ctrl+c` | quit |

### Results

After each test: WPM, raw WPM, accuracy, consistency, WPM graph over time, and character breakdown (correct/incorrect/extra/missed).
