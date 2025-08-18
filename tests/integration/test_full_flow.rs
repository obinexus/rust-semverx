use rust_semverx::{OBINexusSemverX, Version, DependencyResolver, SemverXResolver};

#[test]
fn test_end_to_end_resolution() {
    let mut resolver = SemverXResolver::new();
    
    // Add packages
    resolver.add_package(rust_semverx::resolver::Component {
        name: "libpolycall".to_string(),
        version: "1.0.0".to_string(),
        dependencies: vec![],
    });
    
    resolver.add_package(rust_semverx::resolver::Component {
        name: "node-zero".to_string(),
        version: "2.0.0".to_string(),
        dependencies: vec![],
    });
    
    // Add dependency
    resolver.add_dependency("libpolycall", "node-zero");
    
    // Resolve
    let result = resolver.resolve_dependencies("libpolycall", "1.0.0");
    assert!(result.is_ok());
    
    let deps = result.unwrap();
    assert!(!deps.is_empty());
}

#[test]
fn test_hotwire_integration() {
    // Test the hotwire bypass functionality
    let version = Version::new(1, 0, 0);
    let semverx = OBINexusSemverX::new(version)
        .with_security(rust_semverx::core::SecurityMode::ZeroTrust);
    
    // Simulate hotwire bypass
    let legacy_path = "legacy_service_v0.1";
    let normalized = semverx.normalize_unicode_path(legacy_path);
    
    assert!(!normalized.is_empty());
}
