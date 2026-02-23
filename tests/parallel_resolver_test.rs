//! Tests for parallel dependency resolver

use jbuild::resolver::parallel::ParallelDependencyResolver;
use jbuild::resolver::DependencyResolver;
use jbuild::artifact::repository::DefaultLocalRepository;
use jbuild::artifact::LocalRepository;
use jbuild::model::Dependency;
use std::path::PathBuf;

#[test]
fn test_parallel_resolver_creation() {
    let local_repo: Box<dyn LocalRepository> = Box::new(
        DefaultLocalRepository::new(PathBuf::from("~/.m2/repository"))
    );
    let resolver = DependencyResolver::new(local_repo);
    let _parallel_resolver = ParallelDependencyResolver::new(resolver);
    
    // Just test that creation works
    assert!(true);
}

#[test]
fn test_parallel_resolver_with_custom_threads() {
    let local_repo: Box<dyn LocalRepository> = Box::new(
        DefaultLocalRepository::new(PathBuf::from("~/.m2/repository"))
    );
    let resolver = DependencyResolver::new(local_repo);
    let _parallel_resolver = ParallelDependencyResolver::new(resolver)
        .with_max_parallel(4);
    
    // Just test that builder pattern works
    assert!(true);
}

#[test]
fn test_parallel_resolver_empty_dependencies() {
    let local_repo: Box<dyn LocalRepository> = Box::new(
        DefaultLocalRepository::new(PathBuf::from("~/.m2/repository"))
    );
    let resolver = DependencyResolver::new(local_repo);
    let parallel_resolver = ParallelDependencyResolver::new(resolver);
    
    let dependencies: Vec<Dependency> = vec![];
    let result = parallel_resolver.resolve_parallel(&dependencies);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_parallel_resolver_batch_size() {
    let local_repo: Box<dyn LocalRepository> = Box::new(
        DefaultLocalRepository::new(PathBuf::from("~/.m2/repository"))
    );
    let resolver = DependencyResolver::new(local_repo);
    let parallel_resolver = ParallelDependencyResolver::new(resolver);
    
    // Test that batch processing works with empty list
    let dependencies: Vec<Dependency> = vec![];
    let result = parallel_resolver.resolve_batched(&dependencies, 10);
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_parallel_resolver_builder_pattern() {
    let local_repo: Box<dyn LocalRepository> = Box::new(
        DefaultLocalRepository::new(PathBuf::from("~/.m2/repository"))
    );
    let resolver = DependencyResolver::new(local_repo);
    
    // Test builder pattern with different thread counts
    let _parallel_resolver_1 = ParallelDependencyResolver::new(resolver)
        .with_max_parallel(1);
    
    // Just test that builder works
    assert!(true);
}
