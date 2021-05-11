CC=clang
CFLAGS=-lncurses

BIN=typingtest

all: clean $(BIN)

typingtest: typingtest.c frame.o strlist.o
	$(CC) $^ -o $@ $(CFLAGS)

.PHONY: clean

clean:
	rm -f *.o $(BIN)
