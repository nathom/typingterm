#ifndef FRAME_H
#define FRAME_H

#include "strlist.h"

typedef struct {
    int x0;
    int y0;
    int x1;
    int y1;
} rect_t;

// prototypes
void init_frame();
void draw_rect(rect_t *);
void write_text(char *, rect_t *);
int write_strlist(strlist *bank, rect_t *r, int start_index, int end_index);
void refresh_frame();
void del_frame();

#endif
