//! Tests for persistent build cache

use jbuild::core::persistent_cache::PersistentBuildCache;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_persistent_cache_creation() {
    let cache = PersistentBuildCache::new("test-project".to_string());
    
    assert_eq!(cache.project_id, "test-project");
    assert_eq!(cache.compilation_cache.len(), 0);
    assert_eq!(cache.dependency_cache.len(), 0);
    assert_eq!(cache.test_cache.len(), 0);
}

#[test]
fn test_persistent_cache_needs_compilation() {
    let cache = PersistentBuildCache::new("test-project".to_string());
    let source = PathBuf::from("src/main/java/App.java");
    
    // Initially needs compilation (not in cache and file doesn't exist)
    assert!(cache.needs_compilation(&source));
}

#[test]
fn test_persistent_cache_save_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path();
    
    // Create cache
    let cache = PersistentBuildCache::new("test-project".to_string());
    
    // Save cache
    let save_result = cache.save(cache_dir);
    assert!(save_result.is_ok());
    
    // Load cache
    let loaded = PersistentBuildCache::load(cache_dir, "test-project");
    assert!(loaded.is_ok());
    
    let loaded_cache = loaded.unwrap();
    assert_eq!(loaded_cache.project_id, "test-project");
}

#[test]
fn test_persistent_cache_empty_caches() {
    let cache = PersistentBuildCache::new("test-project".to_string());
    
    // Verify all caches start empty
    assert!(cache.compilation_cache.is_empty());
    assert!(cache.dependency_cache.is_empty());
    assert!(cache.test_cache.is_empty());
}

#[test]
fn test_persistent_cache_version() {
    let cache = PersistentBuildCache::new("test-project".to_string());
    
    // Version should be set
    assert!(!cache.version.is_empty());
}

#[test]
fn test_persistent_cache_load_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path();
    
    // Load from non-existent cache should create new one
    let loaded = PersistentBuildCache::load(cache_dir, "nonexistent");
    assert!(loaded.is_ok());
    
    let cache = loaded.unwrap();
    assert_eq!(cache.project_id, "nonexistent");
}

#[test]
fn test_persistent_cache_multiple_projects() {
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path();
    
    // Create and save multiple project caches
    let cache1 = PersistentBuildCache::new("project1".to_string());
    let cache2 = PersistentBuildCache::new("project2".to_string());
    
    assert!(cache1.save(cache_dir).is_ok());
    assert!(cache2.save(cache_dir).is_ok());
    
    // Load them back
    let loaded1 = PersistentBuildCache::load(cache_dir, "project1");
    let loaded2 = PersistentBuildCache::load(cache_dir, "project2");
    
    assert!(loaded1.is_ok());
    assert!(loaded2.is_ok());
    assert_eq!(loaded1.unwrap().project_id, "project1");
    assert_eq!(loaded2.unwrap().project_id, "project2");
}
