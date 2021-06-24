/* Plan:
 *
 * global rect structs - one for the main screen, the other for
 * the text box to type in.
 *
 * array of rect structs
 * function that draws the rects, according to the screensize (no refresh)
 * open a thread that runs the function every 1/60th of a second
 */

#define _XOPEN_SOURCE_EXTENDED

#include "frame.h"

#include <locale.h>
#include <ncurses.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <wchar.h>

#define NUM_THREADS 1

// constants
const wchar_t HORIZONTAL = 0x2501;
const wchar_t VERTICAL = 0x2503;
const wchar_t CORNERS[] = { 0x250f, 0x2513, 0x251b, 0x2517 };

void* update_bounds();

// globals
int max_y, max_x;

pthread_t threads[NUM_THREADS];

/**
 * Write text to the given rectangle. If the text is too long, it will return.
 *
 * @param text The text to write
 * @param r The rectangle in which to write the text. Must have height >= 3
 * to work properly.
 * @return void
 */
void write_text(char* text, rect_t* r)
{
    int c;
    char* w = text;

    int y, x;
    for (y = r->y0 + 1; y < r->y1; y++)
        for (x = r->x0 + 1; x < r->x1; x++)
            mvaddch(y, x, ' ');

    y = r->y0 + 1, x = r->x0 + 1;
    move(y, x);
    while ((c = *w++)) {
        if (c == '\n')
            move(y + 1, r->x0 + 1);
        else
            addch(c);
        getyx(stdscr, y, x);
        if (x == r->x1) {
            x = r->x0 + 1;
            y++;
            move(y, x);

            if (y == r->y1)
                return;
        }
    }
}

int write_strlist(string* bank, rect_t* r, int start_index, int end_index)
{
    printf("%d, %d, %d, %d\n", r->x0, r->y0, r->x1, r->y1);
    // Clear box
    for (int y0 = r->y0 + 1; y0 < r->y1; y0++) {
        move(y0, r->x0 + 1);
        for (int i = r->x0 + 1; i < r->x1 - 1; i++)
            addch(' ');
    }

    string* curr;
    int counter = start_index;

    // Write characters
    move(r->y0 + 1, r->x0 + 1);
    int wc = 0, y, x; // word count at first line
    for (curr = get_string(bank, start_index); curr->next != NULL;
         curr = curr->next) {
        getyx(stdscr, y, x);
        /* printf("%d, %d\n", y, x); */

        if (curr->len + 1 >= r->x1 - x) {
            x = r->x0 + 1;
            y++;
            move(y, x);
            if (y == r->y1)
                break;
        }

        if (y - r->y0 - 1 == 0)
            wc++;

        attron(curr->style);
        addstr(curr->val);
        attroff(curr->style);

        addch(' ');
        if (counter++ == end_index)
            break;
    }

    return wc;
}

void set_color(int y, int x, int code)
{
    chtype curr = mvinch(y, x);
    mvaddch(y, x, curr | COLOR_PAIR(code));
}

/**
 * Draw a rectangle to the screen.
 *
 * @param r The rect_t struct to draw
 */
void draw_rect(rect_t* r)
{
    int width = r->x1 - r->x0 + 1;
    wchar_t hor_line[width + 1];

    // make horizontal lines for top and bottom
    hor_line[0] = CORNERS[0];
    for (int i = 1; i < width - 1; i++)
        hor_line[i] = HORIZONTAL;
    hor_line[width - 1] = CORNERS[1];
    hor_line[width] = L'\0';

    mvaddwstr(r->y0, r->x0, hor_line);

    hor_line[0] = CORNERS[3];
    hor_line[width - 1] = CORNERS[2];
    mvaddwstr(r->y1, r->x0, hor_line);

    cchar_t vertical_bar;
    setcchar(&vertical_bar, &VERTICAL, 0, 0, NULL);
    for (int y = r->y0 + 1; y < r->y1; y++) {
        mvadd_wch(y, r->x0, &vertical_bar);
        mvadd_wch(y, r->x1, &vertical_bar);
    }
}

void* update_bounds()
{
    for (;;) {
        getmaxyx(stdscr, max_y, max_x);
        usleep(100000);
    }
}

void init_frame()
{
    setlocale(LC_ALL, "");
    initscr();
    noecho();
    start_color();
    use_default_colors();
    curs_set(0);

    // create colors
    init_pair(1, COLOR_GREEN, -1);
    init_pair(2, COLOR_WHITE, COLOR_RED);

    // have the bounds update in the background
    getmaxyx(stdscr, max_y, max_x);
    /* pthread_create(&threads[0], NULL, update_bounds, NULL); */
}

void del_frame()
{
    endwin();
    for (int i = 0; i < NUM_THREADS; i++)
        pthread_cancel(threads[i]);
}
