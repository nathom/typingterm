# if you type 'make' without arguments, this is the default
PROG    = tterm
all:    $(PROG)

# Tell make about the file dependencies
HEAD	= strlist.h frame.h
OBJ     = strlist.o frame.o strlist_test.o typingtest.o

# special libraries This can be blank
LIB     = -lncurses

# select the compiler and flags
# you can over-ride these on the command line e.g. make DEBUG= 
CC      = clang
DEBUG	= -g -fsanitize=address
CSTD	=
WARN	= -Wall -Wextra -Werror
CDEFS	=
CFLAGS	= -I. $(DEBUG) $(WARN) $(CSTD) $(CDEFS)

$(OBJ):	$(HEAD)

# specify how to compile the target
$(PROG):	$(OBJ)
	$(CC) $(CFLAGS) $(OBJ) $(LIB) -o $@


.PHONY: clean test_strlist install

clean:
	rm -f $(OBJ) $(BIN)

install: $(PROG)
	echo "Linking binaries\n"
	rm -f /usr/local/bin/typingterm
	ln ./typingterm /usr/local/bin/typingterm
	rm -f /usr/local/bin/tterm
	ln ./typingterm /usr/local/bin/tterm

strlist_test: $(OBJ)
	$(CC) $^ -o $@ $(CFLAGS)
