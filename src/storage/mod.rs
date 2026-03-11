//! Storage layer – JSON file persistence with atomic writes, backup, and recovery.
//!
//! This module provides:
//! - Platform-specific data directory resolution (`paths`)
//! - Error types for all storage failures (`error`)
//! - Atomic file writing with temp → rename strategy (`atomic_writer`)
//! - Daily backup creation, retention, and recovery (`backup`)
//! - JSON load/save for macros, config, and stats (`json_loader`)
//! - High-level `StorageManager` orchestrating all operations (`macro_repository`)

pub mod error;
pub mod paths;
pub mod atomic_writer;
pub mod backup;
pub mod json_loader;
pub mod macro_repository;
