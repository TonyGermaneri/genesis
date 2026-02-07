// Helper C functions for cubiomes-sys Rust bindings.
// Provides accessors that are hard to replicate via manual FFI.

#include "generator.h"
#include "finders.h"
#include "util.h"
#include <stdlib.h>
#include <string.h>
#include <math.h>

// Forward-declare cubiomes internal functions we need
extern float getSpline(const Spline *sp, const float *vals);

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

// ============================================================================
// Block-level (1:1) height sampling for MC 1.18+
// ============================================================================

// Sample terrain height at true block-level resolution (1 block = 1 sample).
// Coordinates bx, bz are in block coordinates.
// Internally converts to 1:4 biome-noise coordinates (bx/4.0, bz/4.0)
// and replicates the depth calculation from sampleBiomeNoise.
// The resulting height matches mapApproxHeight's output scale.
// Returns 0 on success, 1 if not supported (e.g. wrong MC version or dim).
int cubiomes_map_block_height(float *y,
    const Generator *g,
    int bx, int bz, int w, int h)
{
    if (g->dim != DIM_OVERWORLD)
        return 1;
    if (g->mc < MC_1_18)
        return 1;

    const BiomeNoise *bn = &g->bn;
    int i, j;
    for (j = 0; j < h; j++)
    {
        for (i = 0; i < w; i++)
        {
            // Convert block coords to biome-noise (1:4) coordinates
            double x = (bx + i) / 4.0;
            double z = (bz + j) / 4.0;

            // Apply coordinate shift (same as sampleBiomeNoise)
            double px = x + sampleDoublePerlin(&bn->climate[NP_SHIFT], x, 0, z) * 4.0;
            double pz = z + sampleDoublePerlin(&bn->climate[NP_SHIFT], z, x, 0) * 4.0;

            // Sample the three noise parameters needed for depth spline
            float c = sampleDoublePerlin(&bn->climate[NP_CONTINENTALNESS], px, 0, pz);
            float e = sampleDoublePerlin(&bn->climate[NP_EROSION], px, 0, pz);
            float w_noise = sampleDoublePerlin(&bn->climate[NP_WEIRDNESS], px, 0, pz);

            // Compute PV (peaks and valleys) from weirdness
            float np_param[] = {
                c, e,
                -3.0f * (fabsf(fabsf(w_noise) - 0.6666667f) - 0.33333334f),
                w_noise,
            };

            // Get terrain offset from the depth spline
            double off = getSpline(bn->sp, np_param) + 0.015;

            // Compute depth value (same formula as sampleBiomeNoise at y=0)
            float d = 1.0f - 83.0f / 160.0f + (float)off;

            // Convert to height using same scale as mapApproxHeight
            // mapApproxHeight returns np[NP_DEPTH] / 76.0
            // where np[NP_DEPTH] = (int64_t)(10000.0 * d)
            y[j * w + i] = (10000.0f * d) / 76.0f;
        }
    }
    return 0;
}
