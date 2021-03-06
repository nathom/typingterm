CC=clang
CFLAGS=-lncurses -Ofast

BIN=typingterm

all: clean $(BIN)

typingterm: src/typingtest.c src/frame.o src/strlist.o
	$(CC) $^ -o $@ $(CFLAGS)

.PHONY: clean

clean:
	rm -f *.o $(BIN)

install: typingterm
	echo "Linking binaries\n"
	rm -f /usr/local/bin/typingterm
	ln ./typingterm /usr/local/bin/typingterm
	rm -f /usr/local/bin/tterm
	ln ./typingterm /usr/local/bin/tterm
