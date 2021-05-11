#include "strlist.h"

typedef struct {
    double width_p;   // width = width_p * max_x
    int x;
    int y0;
    int y1;
    int from_bottom;  // if 1, bounds are {max_y + y0, max_y + y1}
} rect_t;

// prototypes
void init_frame();
void draw_rect(rect_t *);
void write_text(char *, rect_t *);
void write_strlist(string *bank, rect_t *r, int start_index, int end_index);
void refresh_frame();
void del_frame();
