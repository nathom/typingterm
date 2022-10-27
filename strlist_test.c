#include <assert.h>
#include <math.h>
#include <stdio.h>

#include "strlist.h"

int main() {
    strlist s = strlist_new(2);
    for (int i = 0; i < 100; i++) {
        strlist_append(&s, (string){ NULL, i, 0 });
        assert(s.cap == (1 << (1 + ((int)log2(i)))));
        assert(s.len <= s.cap);
    }
    for (int i = 0; i < 100; i++) {
        string *a = strlist_get(&s, i);
        assert(a->len == i);
    }

    static int t[100];
    strlist_shuffle(&s);
    for (int i = 0; i < 100; i++) {
        int l = strlist_get(&s, i)->len;
        assert(t[l] == 0);
        t[l] = 1;
    }
    for (int i = 0; i < 100; i++) {
        strlist_set(&s, i, (string){ NULL, i * 2, 0 });
    }
    for (int i = 0; i < 100; i++) {
        assert(strlist_get(&s, i)->len == i * 2);
    }


    printf("All tests passed!\n");
}
