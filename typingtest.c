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

    /* 
     * full width
     * height: 0->6
     * from top
     */
    rect_t main_box = {1.0, 0, 0, 6, 0};
    /*
     * full width
     * height max_y - 3 -> max_y -1
     * from bottom
     */
    rect_t text_box = {1.0, 0, -3, -1, 1};

    // draw boxes
    init_frame();
    draw_rect(&main_box);
    draw_rect(&text_box);

    // shuffle and draw words
    shuffle_strlist(word_bank);


    int c;
    int word_count = 0, words_in_first_line, offset = 0;
    char typed_word[20];
    int cp = 0;  // cursor position

    words_in_first_line = write_strlist(word_bank, &main_box, offset, -1);

    string *curr_word = word_bank->next;
    curr_word->style = A_BOLD;
    // 27 == ESC
    while ((c = getch()) != 27) {
        switch (c) {
            case 127:  // backspace
                if (cp > 0) {
                    typed_word[cp] = '\0';
                    typed_word[--cp] = '\0';
                }
                break;

            case ' ':  // space
                if (strcmp(curr_word->val, typed_word) == 0)
                    curr_word->style = COLOR_PAIR(1);
                else
                    curr_word->style = COLOR_PAIR(2);

                cp = 0;
                typed_word[cp] = '\0';

                curr_word = curr_word->next;
                curr_word->style = A_BOLD;

                word_count++;
                break;

            case '\n':
                // this messes up the boxes
                break;

            default:
                typed_word[cp++] = c;
                typed_word[cp] = '\0';
        }

        write_text(typed_word, &text_box);
        words_in_first_line = write_strlist(word_bank, &main_box, offset, -1);
        /* printf("%d\n", words_in_first_line); */
        if (word_count == words_in_first_line) {
            offset += word_count;
            word_count = 0;
        }
    }

    // exit
    del_frame();
    free_strlist(word_bank);
    // XXX: FREE TEXT BOX

    return 0;
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
