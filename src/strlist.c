/*
 * A linked list of strings.
 */

#include "strlist.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

string *new_string() {
    string *str = malloc(sizeof(string));
    str->len = 0;
    str->next = NULL;
    str->style = 0; // normal
    return str;
}

void insert_string(string *str, char *val, int index) {
    string *before_insert = get_string(str, index - 1);
    string *after_insert = before_insert->next;
    before_insert->next = new_string();
    before_insert->next->val = val;
    before_insert->next->len = strlen(val);
    before_insert->next->next = after_insert;
    str->len++;
}

void append_string(string *str, char *val) {
    string *curr_item;

    for (curr_item = str; curr_item->next != NULL; curr_item = curr_item->next)
        ;

    curr_item->next = new_string();
    curr_item->next->val = val;
    curr_item->next->len = strlen(val);
    str->len++;
}

int set_string(string *str, char *val, int index) {
    string *curr = get_string(str, index);
    curr->val = val;
    curr->len = strlen(val);

    return 0;
}

int delete_string(string *str, int index) {
    string *before = get_string(str, index - 1);
    string *after = before->next->next;
    free_string(before->next);
    before->next = after;

    str->len--;
    return 0;
}

void free_string(string *str) {
    free(str->val);
    free(str);
}

void free_strlist(string *str) {
    string *curr, *prev;
    prev = str;
    for (curr = prev->next; curr->next != NULL; curr = curr->next) {
        free_string(prev);
        prev = curr;
    }
    free_string(prev); // the last string
}

string *get_string(string *str, int index) {
    int target_index;

    if (index < 0)
        target_index = str->len + index;
    else
        target_index = index;

    int i;
    string *curr_item;
    for (i = 0, curr_item = str; i < target_index + 1;
         i++, curr_item = curr_item->next)
        if (curr_item->next == NULL) { // length of list < i
            printf("Error: index %d >= length of list.\n", index);
            exit(1);
        }

    return curr_item;
}

void print_string(string *str) {
    printf("String(val=%s, len=%d)\n", str->val, str->len);
}

void print_strlist(string *str) {
    // inefficient
    for (int i = 0; i < str->len; i++)
        print_string(get_string(str, i));
}

/**
 * Shuffle the strlist.
 *
 * for string in strlist:
 *     r = random int from 0 to len(strlist)-1
 *     swap(string, strlist[r])
 */
void shuffle_strlist(string *str) {
    srand(getpid());
    string *curr, *other;
    char *other_val;
    int other_len;

    int rand_pos;
    for (curr = str->next; curr->next != NULL; curr = curr->next) {
        rand_pos = (double)str->len * rand() / RAND_MAX;
        other = get_string(str, rand_pos);

        other_val = other->val;
        other_len = other->len;

        other->val = curr->val;
        other->len = curr->len;

        curr->val = other_val;
        curr->len = other_len;
    }
}
