#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <ncurses.h>

#include "frame.h"


const char word_bank_file_path[] = "200_top_words.txt";
const char word_delimiter = '\n';

void load_word_bank(string *bank, FILE *bank_file);
void load_text_from_bank(char *text, string *word_bank, int start_index, int end_index);

int main()
{
    string *word_bank = new_string();
    FILE *word_bank_file = fopen(word_bank_file_path, "r");
    load_word_bank(word_bank, word_bank_file);

    int word_bank_size = word_bank->len;
    for (string *curr = word_bank->next; curr->next != NULL; curr = curr->next)
        word_bank_size += curr->len;

    char *text = malloc(word_bank_size);

    /* 
     * full width
     * height: 0->6
     * from top
     */
    rect_t main_box = {1.0, 0, 0, 6, 0};
    /*
     * full width
     * height max_y"hello" - 3 -> max_y -1
     * from bottom
     */
    /* rect_t text_box = {1.0, 0, -3, -1, 1}; */

    init_frame();
    draw_rect(&main_box);

    write_strlist(word_bank, &main_box, 0, 40);

    write_text(text, &main_box);
    refresh();
    getch();
    del_frame();
}

/**
 * @param text The array to load
 * @param word_bank The bank from which to get words
 * @param start_index The index to start at
 * @param end_index The index to end at. Use -1 for None.
 */
void load_text_from_bank(char *text, string *word_bank, int start_index, int end_index)
{
    int i = 0, counter = start_index;
    string *curr;
    for (curr = get_string(word_bank, start_index);
         curr->next != NULL;
         curr = curr->next) 
    {
        strcpy(text + i, curr->val);
        i += curr->len;
        text[i++] = ' ';

        if (counter++ == end_index)
            break;
    }
    text[i] = '\0';
}

void load_word_bank(string *bank, FILE *bank_file)
{
    int c, i = 0;
    const int MAX_WORD_LEN = 20;
    char *word = malloc(MAX_WORD_LEN);
    while ((c = fgetc(bank_file)) != EOF) {
        if (c == word_delimiter) {
            word[i] = '\0'; i = 0;
            append_string(bank, word);
            word = malloc(MAX_WORD_LEN);
        } else 
            word[i++] = c;
    }

}
