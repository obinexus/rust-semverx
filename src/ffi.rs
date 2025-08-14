//! Foreign Function Interface for polyglot interoperability
//! Enables OBINexus components to interface with riftlang, gosilang, and other languages

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::core::{SemverX, VersionConstraint};
use crate::resolver::DependencyResolver;

/// C-compatible version structure for FFI
#[repr(C)]
pub struct CSemverX {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: *const c_char,
    pub build_metadata: *const c_char,
}

impl CSemverX {
    /// Convert from Rust SemverX to C-compatible structure
    pub fn from_rust(version: &SemverX) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            prerelease: version.prerelease.as_ref()
                .map(|s| CString::new(s.as_str()).unwrap().into_raw() as *const c_char)
                .unwrap_or(ptr::null()),
            build_metadata: version.build_metadata.as_ref()
                .map(|s| CString::new(s.as_str()).unwrap().into_raw() as *const c_char)
                .unwrap_or(ptr::null()),
        }
    }
    
    /// Convert from C-compatible structure to Rust SemverX
    pub unsafe fn to_rust(&self) -> SemverX {
        let mut version = SemverX::new(self.major, self.minor, self.patch);
        
        if !self.prerelease.is_null() {
            let c_str = CStr::from_ptr(self.prerelease);
            version.prerelease = Some(c_str.to_string_lossy().into_owned());
        }
        
        if !self.build_metadata.is_null() {
            let c_str = CStr::from_ptr(self.build_metadata);
            version.build_metadata = Some(c_str.to_string_lossy().into_owned());
        }
        
        version
    }
}

/// Free a C-allocated version structure
#[no_mangle]
pub unsafe extern "C" fn semverx_free(version: *mut CSemverX) {
    if version.is_null() {
        return;
    }
    
    let v = &*version;
    
    if !v.prerelease.is_null() {
        let _ = CString::from_raw(v.prerelease as *mut c_char);
    }
    
    if !v.build_metadata.is_null() {
        let _ = CString::from_raw(v.build_metadata as *mut c_char);
    }
    
    // Assuming the CSemverX itself was allocated via Box
    let _ = Box::from_raw(version);
}

/// Parse a version string from C
#[no_mangle]
pub unsafe extern "C" fn semverx_parse(version_str: *const c_char) -> *mut CSemverX {
    if version_str.is_null() {
        return ptr::null_mut();
    }
    
    let c_str = CStr::from_ptr(version_str);
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    match rust_str.parse::<SemverX>() {
        Ok(version) => {
            let c_version = CSemverX::from_rust(&version);
            Box::into_raw(Box::new(c_version))
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Compare two versions
#[no_mangle]
pub unsafe extern "C" fn semverx_compare(v1: *const CSemverX, v2: *const CSemverX) -> c_int {
    if v1.is_null() || v2.is_null() {
        return 0;
    }
    
    let rust_v1 = (*v1).to_rust();
    let rust_v2 = (*v2).to_rust();
    
    use std::cmp::Ordering;
    match rust_v1.cmp(&rust_v2) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Check if a version satisfies a constraint
#[no_mangle]
pub unsafe extern "C" fn semverx_satisfies(
    version: *const CSemverX,
    constraint_str: *const c_char
) -> c_int {
    if version.is_null() || constraint_str.is_null() {
        return 0;
    }
    
    let rust_version = (*version).to_rust();
    
    let c_str = CStr::from_ptr(constraint_str);
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    match VersionConstraint::parse(rust_str) {
        Ok(constraint) => {
            if rust_version.satisfies(&constraint) { 1 } else { 0 }
        }
        Err(_) => 0,
    }
}

/// Normalize a Unicode path (for riftlang/gosilang interop)
#[no_mangle]
pub unsafe extern "C" fn normalize_path(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    
    let c_str = CStr::from_ptr(path);
    let rust_str = c_str.to_string_lossy();
    
    let normalized = crate::normalizer::normalize_unicode_path(&rust_str);
    
    match CString::new(normalized) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a C string allocated by this library
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Opaque handle for dependency resolver
pub struct ResolverHandle {
    resolver: DependencyResolver,
}

/// Create a new dependency resolver
#[no_mangle]
pub extern "C" fn resolver_new() -> *mut ResolverHandle {
    let resolver = DependencyResolver::new();
    let handle = Box::new(ResolverHandle { resolver });
    Box::into_raw(handle)
}

/// Free a dependency resolver
#[no_mangle]
pub unsafe extern "C" fn resolver_free(handle: *mut ResolverHandle) {
    if !handle.is_null() {
        let _ = Box::from_raw(handle);
    }
}

/// Add a package to the resolver
#[no_mangle]
pub unsafe extern "C" fn resolver_add_package(
    handle: *mut ResolverHandle,
    name: *const c_char,
    version: *const c_char,
    deps: *const *const c_char,
    deps_count: usize,
) -> c_int {
    if handle.is_null() || name.is_null() || version.is_null() {
        return 0;
    }
    
    let resolver = &mut (*handle).resolver;
    
    let name_str = CStr::from_ptr(name).to_string_lossy().into_owned();
    let version_str = CStr::from_ptr(version).to_string_lossy().into_owned();
    
    let mut dependencies = Vec::new();
    if !deps.is_null() && deps_count > 0 {
        let dep_slice = std::slice::from_raw_parts(deps, deps_count);
        for &dep_ptr in dep_slice {
            if !dep_ptr.is_null() {
                let dep_str = CStr::from_ptr(dep_ptr).to_string_lossy().into_owned();
                dependencies.push(dep_str);
            }
        }
    }
    
    let package = crate::resolver::PackageNode {
        name: name_str,
        version: version_str,
        dependencies,
    };
    
    resolver.add_package(package);
    1
}

/// Resolve dependencies
#[no_mangle]
pub unsafe extern "C" fn resolver_resolve(
    handle: *mut ResolverHandle,
    result: *mut *mut c_char,
    result_count: *mut usize,
) -> c_int {
    if handle.is_null() || result.is_null() || result_count.is_null() {
        return 0;
    }
    
    let resolver = &mut (*handle).resolver;
    
    match resolver.resolve() {
        Ok(order) => {
            let c_strings: Vec<*mut c_char> = order
                .iter()
                .filter_map(|s| CString::new(s.as_str()).ok())
                .map(|cs| cs.into_raw())
                .collect();
            
            *result_count = c_strings.len();
            
            if !c_strings.is_empty() {
                let size = c_strings.len() * std::mem::size_of::<*mut c_char>();
                let ptr = libc::malloc(size) as *mut *mut c_char;
                
                if ptr.is_null() {
                    // Clean up allocated strings
                    for c_str in c_strings {
                        let _ = CString::from_raw(c_str);
                    }
                    return 0;
                }
                
                std::ptr::copy_nonoverlapping(c_strings.as_ptr(), ptr, c_strings.len());
                std::mem::forget(c_strings); // Prevent dropping since we've transferred ownership
                
                *result = ptr;
            } else {
                *result = ptr::null_mut();
            }
            
            1
        }
        Err(_) => 0,
    }
}

/// Free a result array from resolver_resolve
#[no_mangle]
pub unsafe extern "C" fn resolver_free_result(result: *mut *mut c_char, count: usize) {
    if !result.is_null() && count > 0 {
        let slice = std::slice::from_raw_parts_mut(result, count);
        for &mut c_str in slice {
            if !c_str.is_null() {
                let _ = CString::from_raw(c_str);
            }
        }
        libc::free(result as *mut libc::c_void);
    }
}

// External bindings for riftlang and gosilang runtime
extern "C" {
    // These would be implemented by the riftlang/gosilang runtime
    fn rift_register_package(name: *const c_char, version: *const CSemverX) -> c_int;
    fn gosilang_register_package(name: *const c_char, version: *const CSemverX) -> c_int;
}

/// Register a package with the riftlang runtime
#[no_mangle]
pub unsafe extern "C" fn register_with_rift(
    name: *const c_char,
    version: *const CSemverX
) -> c_int {
    // Safety: Assuming riftlang runtime is properly initialized
    rift_register_package(name, version)
}

/// Register a package with the gosilang runtime
#[no_mangle]
pub unsafe extern "C" fn register_with_gosilang(
    name: *const c_char,
    version: *const CSemverX
) -> c_int {
    // Safety: Assuming gosilang runtime is properly initialized
    gosilang_register_package(name, version)
}

// Note: In a real implementation, you'd need to handle the linking
// with actual riftlang/gosilang runtimes. For now, these are stubs
// that demonstrate the FFI interface structure.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_c_version_conversion() {
        let rust_version = SemverX::new(1, 2, 3);
        let c_version = CSemverX::from_rust(&rust_version);
        
        assert_eq!(c_version.major, 1);
        assert_eq!(c_version.minor, 2);
        assert_eq!(c_version.patch, 3);
        assert!(c_version.prerelease.is_null());
        assert!(c_version.build_metadata.is_null());
        
        // Test with metadata
        let mut rust_version = SemverX::new(2, 0, 0);
        rust_version.prerelease = Some("alpha".to_string());
        rust_version.build_metadata = Some("build.123".to_string());
        
        let c_version = CSemverX::from_rust(&rust_version);
        unsafe {
            let converted = c_version.to_rust();
            assert_eq!(converted.prerelease, Some("alpha".to_string()));
            assert_eq!(converted.build_metadata, Some("build.123".to_string()));
            
            // Clean up
            if !c_version.prerelease.is_null() {
                let _ = CString::from_raw(c_version.prerelease as *mut c_char);
            }
            if !c_version.build_metadata.is_null() {
                let _ = CString::from_raw(c_version.build_metadata as *mut c_char);
            }
        }
    }
}
