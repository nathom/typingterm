#include <stdio.h>
#include <pthread.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>

#include <time.h>
#include <ncurses.h>

#include "frame.h"

#define MAX_WORD_LEN 20
#define NUM_THREADS 2


typedef struct _typingtest_results  {
    double wpm;
    double adj_wpm;
    double time_taken;
    int words_typed;
    double accuracy;
    double cps;  // chars per sec
} typingtest_results;

typedef struct windowsize {
    int x;
    int y;
} windowsize;

void load_word_bank(string *bank, FILE *bank_file, char word_delimiter);
void load_text_from_bank(char *text, string *word_bank, int start_index, int end_index);

// threads
void *update_window_size(void *maxsize);
void *update_time(void *now);

void timef(char *str, long diff);
void update_main_box(rect_t *r, windowsize *size);
void update_text_box(rect_t *r, windowsize *size);
void update_time_box(rect_t *r, windowsize *size);

void print_rect(rect_t *r);

void calculate_stats(typingtest_results *results, long time, string *word_bank,
        string *last_word, int errors);
void show_results(typingtest_results *results, windowsize *);
void final_screen(int timediff, string *word_bank, string *curr_word,
        int num_errors, windowsize *curr_size);
void print_help();
int is_option(char *str, int pos);

const int num_options = 4;
char *options[] = {
    "-t", "--time",
    "-f", "--file",
    "-d", "--delimeter",
    "-h", "--help"
};
char *options_help[] = {
    "Test duration, in seconds. Default 15.",
    "The file containing the word bank. Default 200_top_words.txt.",
    "The character separating words in the file. Default '\\n'.",
    "Show this help message."
};

int main(int argc, char **argv)
{

    int TIMER_LEN = 15;
    char *word_bank_file_path = "200_top_words.txt";
    char word_delimiter = '\n';

    for (int i = 1; i < argc; i++) {
        if (is_option(argv[i], 0))
            TIMER_LEN = atoi(argv[++i]);
        else if (is_option(argv[i], 1))
            word_bank_file_path = argv[++i];
        else if (is_option(argv[i], 2))
            word_delimiter = argv[++i][0];
        else if (is_option(argv[i], 3)) {
            print_help();
            return 1;
        } else {
            printf("Invalid option \"%s\"\n", argv[i]);
            return 1;
        }
    }

    string *word_bank = new_string();
    FILE *word_bank_file = fopen(word_bank_file_path, "r");
    load_word_bank(word_bank, word_bank_file, word_delimiter);
    // shuffle and draw words
    shuffle_strlist(word_bank);

    init_frame();
    windowsize curr_size;
    getmaxyx(stdscr, curr_size.y, curr_size.x);

    rect_t main_box, text_box, time_box;
    update_main_box(&main_box, &curr_size);
    update_text_box(&text_box, &curr_size);
    update_time_box(&time_box, &curr_size);


    // draw boxes
    draw_rect(&main_box);
    draw_rect(&text_box);
    draw_rect(&time_box);


    int c, words_in_first_line, num_errors, word_count, offset;
    char typed_word[MAX_WORD_LEN];
    int cp = 0;  // cursor position
    char timestr[6];  // dd:dd\0
    time_t start, now;

    num_errors = 0;
    word_count = 0;
    offset = 0;


    time(&now);

    pthread_t pids[NUM_THREADS];
    // update time in the background
    pthread_create(&pids[0], NULL, update_time, &now);

    words_in_first_line = write_strlist(word_bank, &main_box, offset, -1);
    /* return 1; */

    string *curr_word = word_bank->next;
    curr_word->style = A_BOLD;
    // 27 == ESC
    c = getch();
    time(&start);
    if (c == 27)
        goto EXIT;

    do {
        timef(timestr, now - start);
        write_text(timestr, &time_box);
        if (now - start >= TIMER_LEN) {
            final_screen(now - start, word_bank, curr_word, num_errors, &curr_size);
            goto EXIT;
        }

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
                else {
                    curr_word->style = COLOR_PAIR(2);
                    num_errors++;
                }

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

    } while ((c = getch()) != 27);

    // exit
    EXIT: {
        del_frame();
        free_strlist(word_bank);
        for (int i = 0; i < NUM_THREADS; i++)
            pthread_cancel(pids[i]);
    }
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

void load_word_bank(string *bank, FILE *bank_file, char word_delimiter)
{
    int c, i = 0;
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

void *update_time(void *now)
{
    while (1) {
        time((time_t *) now);
        usleep(100000);
    }

    return NULL;
}

void timef(char *str, long diff)
{
    int sec = diff % 60;
    int min = diff / 60;
    sprintf(str, "%02d:%02d", min, sec);
}

void *update_window_size(void *maxsize)
{
    windowsize *ms = (windowsize *) maxsize;
    for (;;)
        getmaxyx(stdscr, ms->y, ms->x);
    return NULL;
}

void update_main_box(rect_t *r, windowsize *size)
{
    r->x0 = r->y0 = 0;
    r->x1 = size->x-1;
    r->y1 = size->y * 0.5;
}

void update_text_box(rect_t *r, windowsize *size)
{
    r->x0 = 0;
    r->y0 = size->y - 3;
    r->x1 = size->x * 0.8;
    r->y1 = size->y - 1;
}

void update_time_box(rect_t *r, windowsize *size)
{
    r->x0 = size->x * 0.8 + 1;
    r->y0 = size->y - 3;
    r->x1 = size->x - 1;
    r->y1 = size->y - 1;
}

void print_rect(rect_t *r)
{
    printf("Rectangle(%d, %d, %d, %d)\n", r->x0, r->y0, r->x1, r->y1);
}


void calculate_stats(typingtest_results *results, long time, string *word_bank,
        string *last_word, int errors)
{
    /*
     * double wpm;
    double adj_wpm;
    double time_taken;
    int words_typed;
    double accuracy;
    int cps;  // chars per sec*/

    int words_typed = 0;
    int chars_typed = 0;
    for (
            string *w = word_bank->next;
            w->next != NULL && strcmp(w->val, last_word->val) != 0;
            w = w->next
        ) {
        words_typed++;
        chars_typed += w->len;
    }

    printf("last: %s\n", get_string(word_bank, words_typed)->val);

    results->wpm = 60.0 * words_typed / time;
    results->cps = (double) chars_typed / time;
    results->adj_wpm = 60.0 * (chars_typed / 4.0) / time;
    results->time_taken = time;
    results->words_typed = words_typed;
    if (words_typed > 0.0)
        results->accuracy = (1.0 - (double) errors / words_typed) * 100.0;
    else
        results->accuracy = 0.0;
}

void show_results(typingtest_results *results, windowsize *size)
{
    char result_text[120];
    sprintf(result_text,
            "WPM: %.1f\n"
            "Adjusted WPM: %.1f\n"
            "Time elapsed: %.1f\n"
            "Words typed: %d\n"
            "Accuracy: %.1f%%\n"
            "Chars per second: %.1f",
            results->wpm, results->adj_wpm, results->time_taken,
            results->words_typed, results->accuracy, results->cps);

    rect_t result_box;
    result_box.x0 = size->x / 2 - 11;
    result_box.y0 = size->y * 0.33;
    result_box.x1 = result_box.x0 + 22;
    result_box.y1 = result_box.y0 + 7;
    erase();
    refresh();

    draw_rect(&result_box);
    write_text(result_text, &result_box);
}

void final_screen(int timediff, string *word_bank, string *curr_word,
        int num_errors, windowsize *curr_size)
{
    int c;
    typingtest_results results;
    calculate_stats(&results, timediff, word_bank, curr_word, num_errors);
    show_results(&results, curr_size);
    while ((c = getch()) != 27 && c != 'q')
        ;
}

void print_help()
{
    char spaces[20];
    int num_spaces, j;
    char curr_opt[20];
    printf("Usage: typingterm [OPTIONS]\n\n");
    for (int i = 0; i < num_options; i++) {
        sprintf(curr_opt, "  %s, %s", options[2 * i], options[2 * i + 1]);

        num_spaces = 21 - strlen(curr_opt);
        for (j = 0; j < num_spaces; j++)
            spaces[j] = ' ';
        spaces[j] = '\0';

        printf("%s%s%s\n", curr_opt, spaces, options_help[i]);
    }
}

int is_option(char *str, int pos)
{
    return strcmp(str, options[2 * pos]) == 0 || strcmp(str, options[2 * pos + 1]) == 0;
}
