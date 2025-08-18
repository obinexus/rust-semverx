use rust_semverx::core::{OBINexusSemverX, Version, SecurityMode};

#[test]
fn test_zero_trust_mode() {
    let version = Version::new(1, 0, 0);
    let semverx = OBINexusSemverX::new(version)
        .with_security(SecurityMode::ZeroTrust);
    
    assert_eq!(semverx.security_mode, SecurityMode::ZeroTrust);
}

#[test]
fn test_secure_path_normalization() {
    let version = Version::new(1, 0, 0);
    let semverx = OBINexusSemverX::new(version)
        .with_security(SecurityMode::Hardened);
    
    // Test path traversal prevention
    let malicious_path = "../../../etc/passwd";
    let normalized = semverx.normalize_unicode_path(malicious_path);
    
    // Should normalize but not allow traversal
    assert!(!normalized.contains(".."));
}

#[test]
fn test_secure_version_constraints() {
    // Test that version constraints are properly validated
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.0.0'; DROP TABLE versions;--").unwrap_err();
    
    // SQL injection attempt should fail parsing
    assert!(v2.contains("Invalid"));
}
