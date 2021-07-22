#include <stdio.h>
#include <time.h>
#include <unistd.h>

// create colors

void timef(char *str, struct timespec *start, struct timespec *now);

int main() {
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    sleep(1);
    usleep(123456);
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);

    char stuff[100];
    timef(stuff, &start, &end);
    printf("%s\n", stuff);
}

void timef(char *str, struct timespec *start, struct timespec *now) {
    double diff = now->tv_sec - start->tv_sec + ((double)now->tv_nsec - start->tv_nsec) / 10e8;
    int mins = (int)diff / 60;
    sprintf(str, "%02d:%.2f", mins, diff);
}
