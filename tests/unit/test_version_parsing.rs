use rust_semverx::core::Version;

#[test]
fn test_basic_version() {
    let v = Version::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
}

#[test]
fn test_version_with_prerelease() {
    let v = Version::parse("1.0.0-alpha.1").unwrap();
    assert_eq!(v.pre, Some("alpha.1".to_string()));
}

#[test]
fn test_version_with_build_metadata() {
    let v = Version::parse("1.0.0+build.123").unwrap();
    assert_eq!(v.build, Some("build.123".to_string()));
}

#[test]
fn test_complex_version() {
    let v = Version::parse("2.1.0-rc.1+build.456").unwrap();
    assert_eq!(v.major, 2);
    assert_eq!(v.minor, 1);
    assert_eq!(v.patch, 0);
    assert_eq!(v.pre, Some("rc.1".to_string()));
    assert_eq!(v.build, Some("build.456".to_string()));
}

#[test]
fn test_version_comparison() {
    let v1 = Version::parse("1.0.0").unwrap();
    let v2 = Version::parse("1.0.1").unwrap();
    let v3 = Version::parse("1.0.0-alpha").unwrap();
    
    assert!(v1 < v2);
    assert!(v3 < v1); // pre-release versions are less than release
}

#[test]
fn test_invalid_versions() {
    assert!(Version::parse("").is_err());
    assert!(Version::parse("1").is_err());
    assert!(Version::parse("1.2").is_err());
    assert!(Version::parse("a.b.c").is_err());
}
