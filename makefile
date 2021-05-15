CC=gcc
CFLAGS=-lncurses

BIN=typingtest

all: clean $(BIN)

typingtest: typingtest.c frame.o strlist.o
	$(CC) $^ -o $@ $(CFLAGS)

.PHONY: clean

clean:
	rm -f *.o $(BIN)

install: typingtest
	rm -f /usr/local/bin/typingtest
	ln ./typingtest /usr/local/bin/typingtest
