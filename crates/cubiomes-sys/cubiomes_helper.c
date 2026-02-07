// Helper C functions for cubiomes-sys Rust bindings.
// Provides accessors that are hard to replicate via manual FFI.

#include "generator.h"
#include "finders.h"
#include "util.h"
#include <stdlib.h>
#include <string.h>

// Return sizeof(Generator) so Rust can allocate the right amount of memory.
size_t cubiomes_generator_size(void) {
    return sizeof(Generator);
}

size_t cubiomes_generator_align(void) {
    return _Alignof(Generator);
}

// Allocate a Generator on the heap and return a pointer.
Generator* cubiomes_generator_new(void) {
    Generator *g = (Generator*) calloc(1, sizeof(Generator));
    return g;
}

// Free a heap-allocated Generator.
void cubiomes_generator_free(Generator *g) {
    if (g) free(g);
}

// Wrapper: setup + apply seed in one call.
void cubiomes_generator_init(Generator *g, int mc, uint32_t flags, int dim, uint64_t seed) {
    setupGenerator(g, mc, flags);
    applySeed(g, dim, seed);
}

// Get the mc version from a generator.
int cubiomes_generator_get_mc(const Generator *g) {
    return g->mc;
}

// Get the seed from a generator.
uint64_t cubiomes_generator_get_seed(const Generator *g) {
    return g->seed;
}

// Get the dimension from a generator.
int cubiomes_generator_get_dim(const Generator *g) {
    return g->dim;
}

// ============================================================================
// SurfaceNoise helpers
// ============================================================================

// Return sizeof(SurfaceNoise) so Rust can use opaque allocation.
size_t cubiomes_surface_noise_size(void) {
    return sizeof(SurfaceNoise);
}

size_t cubiomes_surface_noise_align(void) {
    return _Alignof(SurfaceNoise);
}

// Allocate a SurfaceNoise on the heap and return a pointer.
SurfaceNoise* cubiomes_surface_noise_new(void) {
    SurfaceNoise *sn = (SurfaceNoise*) calloc(1, sizeof(SurfaceNoise));
    return sn;
}

// Free a heap-allocated SurfaceNoise.
void cubiomes_surface_noise_free(SurfaceNoise *sn) {
    if (sn) free(sn);
}

// Initialize a SurfaceNoise for a given dimension and seed.
void cubiomes_surface_noise_init(SurfaceNoise *sn, int dim, uint64_t seed) {
    initSurfaceNoise(sn, dim, seed);
}

// Wrapper for mapApproxHeight. Returns surface heights in blocks at 1:4 scale.
// Writes w*h floats into the y buffer.
// Returns 0 on success.
int cubiomes_map_approx_height(float *y, int *ids,
    const Generator *g, const SurfaceNoise *sn,
    int x, int z, int w, int h) {
    return mapApproxHeight(y, ids, g, sn, x, z, w, h);
}
