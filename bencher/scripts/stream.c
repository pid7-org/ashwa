/*
 * STREAM: Sustainable Memory Bandwidth in High Performance Computers
 * Simple C version for baseline memory bandwidth measurement.
 */

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <float.h>
#include <sys/time.h>

#ifndef STREAM_ARRAY_SIZE
#   define STREAM_ARRAY_SIZE 10000000
#endif

#ifndef NTIMES
#   define NTIMES 10
#endif

#ifndef OFFSET
#   define OFFSET 0
#endif

static double a[STREAM_ARRAY_SIZE+OFFSET];
static double b[STREAM_ARRAY_SIZE+OFFSET];
static double c[STREAM_ARRAY_SIZE+OFFSET];

static double mysecond() {
    struct timeval tp;
    gettimeofday(&tp, NULL);
    return ((double)tp.tv_sec + (double)tp.tv_usec * 1.e-6);
}

int main() {
    int quantum, checktick();
    int j, k;
    double scalar, t, times[4][NTIMES];
    double bytes[4];
    
    bytes[0] = 2.0 * sizeof(double) * STREAM_ARRAY_SIZE; // Copy
    bytes[1] = 2.0 * sizeof(double) * STREAM_ARRAY_SIZE; // Scale
    bytes[2] = 3.0 * sizeof(double) * STREAM_ARRAY_SIZE; // Add
    bytes[3] = 3.0 * sizeof(double) * STREAM_ARRAY_SIZE; // Triad

    for (j = 0; j < STREAM_ARRAY_SIZE; j++) {
        a[j] = 1.0;
        b[j] = 2.0;
        c[j] = 0.0;
    }

    scalar = 3.0;
    for (k = 0; k < NTIMES; k++) {
        // Copy
        t = mysecond();
        for (j = 0; j < STREAM_ARRAY_SIZE; j++) c[j] = a[j];
        times[0][k] = mysecond() - t;

        // Scale
        t = mysecond();
        for (j = 0; j < STREAM_ARRAY_SIZE; j++) b[j] = scalar * c[j];
        times[1][k] = mysecond() - t;

        // Add
        t = mysecond();
        for (j = 0; j < STREAM_ARRAY_SIZE; j++) c[j] = a[j] + b[j];
        times[2][k] = mysecond() - t;

        // Triad
        t = mysecond();
        for (j = 0; j < STREAM_ARRAY_SIZE; j++) a[j] = b[j] + scalar * c[j];
        times[3][k] = mysecond() - t;
    }

    // Compute min times (excluding first warmup run)
    double min_triad = DBL_MAX;
    for (k = 1; k < NTIMES; k++) {
        if (times[3][k] < min_triad) min_triad = times[3][k];
    }

    double triad_rate = 1.0E-06 * bytes[3] / min_triad;

    printf("TRIAD_BEST_RATE_MB_S: %.2f\n", triad_rate);
    printf("TRIAD_MIN_TIME_S: %.6f\n", min_triad);

    return 0;
}
