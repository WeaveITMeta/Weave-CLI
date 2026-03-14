// =============================================================================
// Engine Module - Template download, pruning, and config generation
// =============================================================================
//
// Table of Contents:
// - mod.rs: Module declarations
// - downloader.rs: GitHub release fetcher with caching
// - pruner.rs: Directory tree pruner based on selections
// - generator.rs: Config file generator (package.json, docker-compose, etc.)
// =============================================================================

pub mod downloader;
pub mod generator;
pub mod pruner;
