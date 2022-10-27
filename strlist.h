#include <stdint.h>

typedef struct _string {
    char *val;
    int len;
    int style;
} string;

typedef struct _strlist {
    uintptr_t len;
    uintptr_t cap;
    string *vec;
} strlist;

// Constructor/destructor
strlist strlist_new(uintptr_t size);
void strlist_delete(strlist *);

string strlist_get(strlist *s, uintptr_t index);
uintptr_t strlist_set(strlist *l, uintptr_t index, string s);
uintptr_t strlist_append(strlist *l, string s);
void strlist_shuffle(strlist *s);


// string *get_string(string *str, int index);
// string *new_string();
// void insert_string(string *str, char *val, int index);
// void append_string(string *str, char *val);
// void print_string(string *str);
// void print_strlist(string *str);
// int set_string(string *str, char *val, int index);
// int delete_string(string *str, int index);
// void shuffle_strlist(string *);
// void free_string(string *);
// void free_strlist(string *);
// void free_string(string *str);
