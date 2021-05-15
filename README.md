# typingterm

A typingtest for terminal power users.

![screenshot](demo/demo1.png)

## Usage

Ensure you have `ncurses` installed. You can do this with Homebrew

```bash
brew install ncurses
```

Clone this repository

```bash
git clone https://github.com/nathom/typingterm
```

Change into the directory, compile, and link the binary.

```
cd typingterm && make install
```

Run the program with

```
typingterm
```

or

```
tterm
```

If you don't want to link the binary, you can just run `make` with no args.



```bash
❯ tterm -h
Usage: typingterm [OPTIONS]

  -t, --time         Test duration, in seconds. Default 15.
  -f, --file         The file containing the word bank. Default 200_top_words.txt.
  -d, --delimeter    The character separating words in the file. Default '\n'.
  -h, --help         Show this help message.
```
