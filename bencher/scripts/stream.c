/*
 * STREAM Triad baseline memory bandwidth benchmark.
 * NOTE: Used to establish baseline DRAM saturation throughput on the target
 * node.
 */

#include <float.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/time.h>

#ifndef STREAM_ARRAY_SIZE
#define STREAM_ARRAY_SIZE 10000000
#endif

#ifndef NTIMES
#define NTIMES 10
#endif

#ifndef OFFSET
#define OFFSET 0
#endif

static double a[STREAM_ARRAY_SIZE + OFFSET];
static double b[STREAM_ARRAY_SIZE + OFFSET];
static double c[STREAM_ARRAY_SIZE + OFFSET];

static inline double get_wall_time(void) {
  struct timeval tp;
  gettimeofday(&tp, NULL);
  return ((double)tp.tv_sec + (double)tp.tv_usec * 1e-6);
}

int main(void) {
  int j, k;
  double scalar = 3.0;
  double min_triad = DBL_MAX;
  const double bytes_triad = 3.0 * sizeof(double) * (double)STREAM_ARRAY_SIZE;

  // NOTE: Touch pages to ensure physical allocation and warm TLB entries
  for (j = 0; j < STREAM_ARRAY_SIZE; j++) {
    a[j] = 1.0;
    b[j] = 2.0;
    c[j] = 0.0;
  }

  double dummy = 0.0;
  for (k = 0; k < NTIMES; k++) {
    double t0 = get_wall_time();
    for (j = 0; j < STREAM_ARRAY_SIZE; j++) {
      a[j] = b[j] + scalar * c[j];
    }
    double dt = get_wall_time() - t0;
    dummy += a[k % STREAM_ARRAY_SIZE];

    if (k > 0 && dt > 0.0 && dt < min_triad) {
      min_triad = dt;
    }
  }

  if (dummy == 0.0) {
    fprintf(stderr, "Unexpected zero accumulator\n");
  }

  double triad_rate =
      (min_triad > 0.0) ? (1.0e-6 * bytes_triad / min_triad) : 0.0;
  printf("TRIAD_BEST_RATE_MB_S: %.2f\n", triad_rate);
  printf("TRIAD_MIN_TIME_S: %.6f\n", min_triad);

  return 0;
}
