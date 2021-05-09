CC = gcc
CFLAGS = -D_XOPEN_SOURCE_EXTENDED -lncurses
NCURSESW = /usr/local/Cellar/ncurses/6.2/lib/libncursesw.a

frame: frame.c
	$(CC) -o $@ $< $(NCURSESW)
