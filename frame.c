#define _XOPEN_SOURCE_EXTENDED

#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <pthread.h>

#include <wchar.h>
#include <locale.h>

#include <ncurses.h>

int max_x, max_y;

const wchar_t HORIZONTAL = 0x2501;
const wchar_t VERTICAL = 0x2503;
const wchar_t CORNERS[] = {0x250f, 0x2513, 0x251b, 0x2517};

typedef struct {
    int x0;
    int y0;
    int x1;
    int y1;
} rectangle;

void wstrcpy(wchar_t *target, wchar_t *source);
int draw_rect(rectangle *rect);
void offset_draw_rect(rectangle *rect, int *offsets);


int main(int argc, char *argv[])
{
    // initialization
    setlocale(LC_ALL, "");
    initscr();
    noecho();
    start_color();
    use_default_colors();
    curs_set(0);
    getmaxyx(stdscr, max_y, max_x);
    rectangle word_bank = {0, 0, max_x, max_y};

    // main program
    draw_rect(&word_bank);

    refresh();
    getch();

    endwin();
    return 0;
}


/**
 * Draws a rectangle with the top left corner
 * at (x0, y0) and the bottom right corner at
 * (x1, y1).
 */
int draw_rect(rectangle *rect)
{
    int hor_line_len = rect->x1 - rect->x0;
    wchar_t hor_line[hor_line_len+1];
    int i;
    hor_line[0] = CORNERS[0];
    for (i = 1; i < hor_line_len - 1; i++)
        hor_line[i] = HORIZONTAL;

    hor_line[hor_line_len - 1] = CORNERS[1];
    hor_line[hor_line_len] = '\0';

    mvaddwstr(rect->y0, rect->x0, hor_line);

    hor_line[0] = CORNERS[3];
    hor_line[hor_line_len - 1] = CORNERS[2];
    mvaddwstr(rect->y1, rect->x0, hor_line);

    cchar_t vertical_bar;
    setcchar(&vertical_bar, &VERTICAL, 0, 0, NULL);
    for (i = rect->y0 + 1; i < rect->y1; i++) {
        mvadd_wch(i, rect->x0, &vertical_bar);
        mvadd_wch(i, rect->x1-1, &vertical_bar);
    }

    return 0;
}

void update_border(rectangle *rect, int *offsets)
{
    int y, x;
    int prev_y, prev_x;
    prev_y = prev_x = 0;

    for (;;) {
        getmaxyx(stdscr, y, x);
        if (y != prev_y || x != prev_x) {
            offset_draw_rect(rect, offsets);
            prev_y = y;
            prev_x = x;
        }
    }
    usleep(0.1);
}

void offset_draw_rect(rectangle *rect, int *offsets)
{
    rectangle new_rect = {
        rect->x0 + offsets[0],
        rect->y0 + offsets[1],
        rect->x1 + offsets[2],
        rect->y1 + offsets[2],
    };
    draw_rect(&new_rect);
}

void wstrcpy(wchar_t *target, wchar_t *source)
{
    while ((*target++ = *source++))
        ;
}
