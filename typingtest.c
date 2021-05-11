#include <stdio.h>
#include <stdlib.h>

#include "frame.h"
#include "linkedlist.h"


const char word_bank_file_path[] = "200_top_words.txt";
const char word_delimiter = '\n';

void load_word_bank(string *bank, FILE *bank_file);

int main()
{
    string *word_bank = new_string();
    FILE *word_bank_file = fopen(word_bank_file_path, "r");
    load_word_bank(word_bank, word_bank_file);
    print_strlist(word_bank);
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
