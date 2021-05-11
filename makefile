CC=gcc
CFLAGS=-lncurses

BIN=typingtest

all: clean $(BIN)

typingtest: typingtest.c frame.o linkedlist.o
	$(CC) $^ -o $@ $(CFLAGS)

.PHONY: clean

clean:
	rm -f *.o $(BIN)
