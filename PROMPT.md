# Iteration 13: Kernel Agent - Save/Load Infrastructure

## Objective
Implement chunk serialization, region files, compression, and incremental saves.

## Tasks

### 1. Chunk Serialization (chunk_serialize.rs)
- ChunkData binary format (cell states, metadata)
- Version header for format migration
- Efficient encoding for sparse data
- Checksum for corruption detection

### 2. World Region Files (world_region.rs)
- Region file format (32x32 chunks per file)
- Chunk offset table for fast lookup
- Memory-mapped file access option
- Region file creation/loading

### 3. Compression Support (save_compression.rs)
- LZ4 for fast compression/decompression
- Zstd for better ratio (optional)
- Streaming compression for large chunks
- Compression level configuration

### 4. Incremental Saves (incremental_save.rs)
- Dirty chunk tracking
- Delta encoding for changes
- Background save thread
- Save queue management

### 5. Update lib.rs
Export: chunk_serialize, world_region, save_compression, incremental_save
