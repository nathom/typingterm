#define _XOPEN_SOURCE_EXTENDED

#include <ncurses.h>
#include <wchar.h>


const wchar_t VERTICAL = 0x2503;

int main() {
    int h[2] = L"━";
    printf("%d, %d\n%x, %x\n%c, %c\n", h[0], h[1], h[0], h[1], h[0], h[1]);
}
