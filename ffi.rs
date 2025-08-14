// SEMVERX Polyglot FFI Bindings
// Provides C-compatible interface for integration with multiple languages

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_double};
use std::ptr;
use std::slice;
use serde_json;

use crate::{SemverX, BubblingError, VerbNounComponent, VerbNounClass, 
            ComponentHealth, StressZone, SEIMetadata, Environment, Classifier};
use crate::resolver::{SemverXResolver, ResolutionStrategy, ResolutionResult};

// ==================== C-Compatible Types ====================

#[repr(C)]
pub struct CSemverX {
    major: u32,
    minor: u32,
    patch: u32,
    prerelease: *const c_char,
    build: *const c_char,
    environment: *const c_char,
    classifier: *const c_char,
    intent: *const c_char,
    sei_statement_contract: *const c_char,
    sei_statement_version: u32,
    sei_expression_signature: *const c_char,
    sei_expression_complexity: u32,
    sei_intent_hash: *const c_char,
    sei_intent_priority: u32,
}

#[repr(C)]
pub struct CResolutionResult {
    resolved_count: u32,
    resolved_json: *const c_char,
    strategy_used: c_int,
    iterations: u32,
    stress_impact: c_double,
    error_message: *const c_char,
}

#[repr(C)]
pub struct CComponentHealth {
    stress_level: c_double,
    zone: c_int,
    heal_attempts: u32,
    last_heal_timestamp: u64,
    can_self_heal: u8,
}

#[repr(C)]
pub struct CBubblingError {
    error_type: c_int,
    message: *const c_char,
    context_count: u32,
    contexts: *const *const c_char,
    stress_level: c_double,
    can_recover: u8,
}

// ==================== FFI Exports ====================

/// Parse a version string into SemverX structure
#[no_mangle]
pub extern "C" fn semverx_parse(version_str: *const c_char) -> *mut CSemverX {
    if version_str.is_null() {
        return ptr::null_mut();
    }
    
    let c_str = unsafe { CStr::from_ptr(version_str) };
    let version_string = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    match SemverX::parse(version_string) {
        Ok(semver) => {
            let c_semver = Box::new(convert_to_c_semverx(&semver));
            Box::into_raw(c_semver)
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Free a CSemverX structure
#[no_mangle]
pub extern "C" fn semverx_free(semverx: *mut CSemverX) {
    if !semverx.is_null() {
        unsafe {
            let semver = Box::from_raw(semverx);
            free_c_semverx(&*semver);
        }
    }
}

/// Compare two versions
#[no_mangle]
pub extern "C" fn semverx_compare(v1: *const CSemverX, v2: *const CSemverX) -> c_int {
    if v1.is_null() || v2.is_null() {
        return -2; // Error code
    }
    
    unsafe {
        let semver1 = convert_from_c_semverx(&*v1);
        let semver2 = convert_from_c_semverx(&*v2);
        
        match (semver1, semver2) {
            (Ok(s1), Ok(s2)) => s1.compare(&s2),
            _ => -2,
        }
    }
}

/// Check if a version satisfies a range
#[no_mangle]
pub extern "C" fn semverx_satisfies(
    version: *const CSemverX, 
    range: *const c_char
) -> c_int {
    if version.is_null() || range.is_null() {
        return -1;
    }
    
    unsafe {
        let semver = match convert_from_c_semverx(&*version) {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        let range_str = match CStr::from_ptr(range).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        match semver.satisfies(range_str) {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(_) => -1,
        }
    }
}

/// Validate a version according to OBINexus policies
#[no_mangle]
pub extern "C" fn semverx_validate(version: *const CSemverX) -> c_int {
    if version.is_null() {
        return -1;
    }
    
    unsafe {
        let semver = match convert_from_c_semverx(&*version) {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        match semver.validate() {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }
}

// ==================== Resolver FFI ====================

/// Create a new resolver instance
#[no_mangle]
pub extern "C" fn semverx_resolver_new() -> *mut SemverXResolver {
    let resolver = Box::new(SemverXResolver::new());
    Box::into_raw(resolver)
}

/// Free a resolver instance
#[no_mangle]
pub extern "C" fn semverx_resolver_free(resolver: *mut SemverXResolver) {
    if !resolver.is_null() {
        unsafe {
            let _ = Box::from_raw(resolver);
        }
    }
}

/// Resolve dependencies for a component
#[no_mangle]
pub extern "C" fn semverx_resolve(
    resolver: *mut SemverXResolver,
    component_id: *const c_char,
    strategy: c_int,
) -> *mut CResolutionResult {
    if resolver.is_null() || component_id.is_null() {
        return ptr::null_mut();
    }
    
    let component_str = unsafe {
        match CStr::from_ptr(component_id).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let strategy = match strategy {
        0 => Some(ResolutionStrategy::Eulerian),
        1 => Some(ResolutionStrategy::Hamiltonian),
        2 => Some(ResolutionStrategy::AStar),
        3 => Some(ResolutionStrategy::Hybrid),
        _ => None,
    };
    
    let resolver = unsafe { &mut *resolver };
    
    match resolver.resolve(component_str, strategy) {
        Ok(result) => {
            let c_result = Box::new(convert_to_c_resolution_result(&result));
            Box::into_raw(c_result)
        }
        Err(e) => {
            let error_msg = CString::new(format!("{}", e)).unwrap_or_default();
            let c_result = Box::new(CResolutionResult {
                resolved_count: 0,
                resolved_json: ptr::null(),
                strategy_used: -1,
                iterations: 0,
                stress_impact: e.stress_level,
                error_message: error_msg.into_raw(),
            });
            Box::into_raw(c_result)
        }
    }
}

/// Free a resolution result
#[no_mangle]
pub extern "C" fn semverx_resolution_result_free(result: *mut CResolutionResult) {
    if !result.is_null() {
        unsafe {
            let result = Box::from_raw(result);
            if !result.resolved_json.is_null() {
                let _ = CString::from_raw(result.resolved_json as *mut c_char);
            }
            if !result.error_message.is_null() {
                let _ = CString::from_raw(result.error_message as *mut c_char);
            }
        }
    }
}

// ==================== Health Monitoring FFI ====================

/// Create a component health monitor
#[no_mangle]
pub extern "C" fn semverx_health_new() -> *mut CComponentHealth {
    let health = Box::new(CComponentHealth {
        stress_level: 0.0,
        zone: 0,
        heal_attempts: 0,
        last_heal_timestamp: 0,
        can_self_heal: 1,
    });
    Box::into_raw(health)
}

/// Update health stress level
#[no_mangle]
pub extern "C" fn semverx_health_update_stress(
    health: *mut CComponentHealth,
    new_stress: c_double,
) -> c_int {
    if health.is_null() {
        return -1;
    }
    
    unsafe {
        (*health).stress_level = new_stress;
        
        // Update zone based on stress
        (*health).zone = match new_stress {
            s if s < 3.0 => 0,  // Ok
            s if s < 6.0 => 1,  // Warning
            s if s < 9.0 => 2,  // Danger
            _ => 3,             // Critical
        };
        
        // Update self-heal capability
        (*health).can_self_heal = if new_stress < 9.0 && (*health).heal_attempts < 3 {
            1
        } else {
            0
        };
        
        (*health).zone
    }
}

/// Free health monitor
#[no_mangle]
pub extern "C" fn semverx_health_free(health: *mut CComponentHealth) {
    if !health.is_null() {
        unsafe {
            let _ = Box::from_raw(health);
        }
    }
}

// ==================== Language-Specific Bindings ====================

// Python bindings via ctypes
#[no_mangle]
pub extern "C" fn semverx_python_parse_json(json_str: *const c_char) -> *const c_char {
    if json_str.is_null() {
        return ptr::null();
    }
    
    let json = unsafe {
        match CStr::from_ptr(json_str).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null(),
        }
    };
    
    // Parse JSON to SemverX
    match serde_json::from_str::<SemverX>(json) {
        Ok(semver) => {
            match serde_json::to_string(&semver) {
                Ok(result) => {
                    let c_str = CString::new(result).unwrap_or_default();
                    c_str.into_raw()
                }
                Err(_) => ptr::null(),
            }
        }
        Err(_) => ptr::null(),
    }
}

// Node.js bindings via N-API
#[no_mangle]
pub extern "C" fn semverx_node_create_buffer(
    data: *const u8,
    len: usize,
) -> *mut u8 {
    if data.is_null() || len == 0 {
        return ptr::null_mut();
    }
    
    unsafe {
        let slice = slice::from_raw_parts(data, len);
        let mut buffer = Vec::with_capacity(len);
        buffer.extend_from_slice(slice);
        let ptr = buffer.as_mut_ptr();
        std::mem::forget(buffer);
        ptr
    }
}

// PHP bindings via FFI
#[no_mangle]
pub extern "C" fn semverx_php_array_parse(
    keys: *const *const c_char,
    values: *const *const c_char,
    count: usize,
) -> *const c_char {
    if keys.is_null() || values.is_null() || count == 0 {
        return ptr::null();
    }
    
    let mut map = std::collections::HashMap::new();
    
    unsafe {
        let keys_slice = slice::from_raw_parts(keys, count);
        let values_slice = slice::from_raw_parts(values, count);
        
        for i in 0..count {
            if let (Ok(key), Ok(value)) = (
                CStr::from_ptr(keys_slice[i]).to_str(),
                CStr::from_ptr(values_slice[i]).to_str(),
            ) {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    match serde_json::to_string(&map) {
        Ok(json) => {
            let c_str = CString::new(json).unwrap_or_default();
            c_str.into_raw()
        }
        Err(_) => ptr::null(),
    }
}

// TypeScript/JavaScript type definitions generator
#[no_mangle]
pub extern "C" fn semverx_generate_typescript_defs() -> *const c_char {
    let defs = r#"
export interface SemverX {
    major: number;
    minor: number;
    patch: number;
    prerelease?: string;
    build?: string;
    environment?: 'dev' | 'test' | 'staging' | 'prod';
    classifier?: 'stable' | 'legacy' | 'experimental' | 'deprecated';
    intent?: string;
    sei: SEIMetadata;
}

export interface SEIMetadata {
    statementContract: string;
    statementVersion: number;
    expressionSignature: string;
    expressionComplexity: number;
    intentHash: string;
    intentPriority: number;
}

export interface ComponentHealth {
    stressLevel: number;
    zone: 'ok' | 'warning' | 'danger' | 'critical';
    healAttempts: number;
    lastHealTimestamp: number;
    canSelfHeal: boolean;
}

export interface ResolutionResult {
    resolvedVersions: Map<string, SemverX>;
    strategyUsed: 'eulerian' | 'hamiltonian' | 'astar' | 'hybrid';
    iterations: number;
    stressImpact: number;
}

export class SemverXNative {
    static parse(version: string): SemverX | null;
    static compare(v1: SemverX, v2: SemverX): number;
    static satisfies(version: SemverX, range: string): boolean;
    static validate(version: SemverX): boolean;
    static resolve(componentId: string, strategy?: string): Promise<ResolutionResult>;
}
"#;
    
    let c_str = CString::new(defs).unwrap_or_default();
    c_str.into_raw()
}

// ==================== Helper Functions ====================

fn convert_to_c_semverx(semver: &SemverX) -> CSemverX {
    CSemverX {
        major: semver.major,
        minor: semver.minor,
        patch: semver.patch,
        prerelease: to_c_string_ptr(&semver.prerelease),
        build: to_c_string_ptr(&semver.build),
        environment: to_c_string_ptr(&semver.environment.as_ref().map(|e| format!("{:?}", e))),
        classifier: to_c_string_ptr(&semver.classifier.as_ref().map(|c| format!("{:?}", c))),
        intent: to_c_string_ptr(&semver.intent),
        sei_statement_contract: to_c_string(&semver.sei.statement_contract),
        sei_statement_version: semver.sei.statement_version,
        sei_expression_signature: to_c_string(&semver.sei.expression_signature),
        sei_expression_complexity: semver.sei.expression_complexity,
        sei_intent_hash: to_c_string(&semver.sei.intent_hash),
        sei_intent_priority: semver.sei.intent_priority,
    }
}

fn convert_from_c_semverx(c_semver: &CSemverX) -> Result<SemverX, String> {
    Ok(SemverX {
        major: c_semver.major,
        minor: c_semver.minor,
        patch: c_semver.patch,
        prerelease: from_c_string_ptr(c_semver.prerelease),
        build: from_c_string_ptr(c_semver.build),
        environment: from_c_string_ptr(c_semver.environment)
            .and_then(|s| match s.as_str() {
                "Dev" => Some(Environment::Dev),
                "Test" => Some(Environment::Test),
                "Staging" => Some(Environment::Staging),
                "Prod" => Some(Environment::Prod),
                _ => None,
            }),
        classifier: from_c_string_ptr(c_semver.classifier)
            .and_then(|s| match s.as_str() {
                "Stable" => Some(Classifier::Stable),
                "Legacy" => Some(Classifier::Legacy),
                "Experimental" => Some(Classifier::Experimental),
                "Deprecated" => Some(Classifier::Deprecated),
                _ => None,
            }),
        intent: from_c_string_ptr(c_semver.intent),
        sei: SEIMetadata {
            statement_contract: from_c_string_ptr(c_semver.sei_statement_contract)
                .unwrap_or_default(),
            statement_version: c_semver.sei_statement_version,
            expression_signature: from_c_string_ptr(c_semver.sei_expression_signature)
                .unwrap_or_default(),
            expression_complexity: c_semver.sei_expression_complexity,
            intent_hash: from_c_string_ptr(c_semver.sei_intent_hash)
                .unwrap_or_default(),
            intent_priority: c_semver.sei_intent_priority,
        },
    })
}

fn convert_to_c_resolution_result(result: &ResolutionResult) -> CResolutionResult {
    let json = serde_json::to_string(&result.resolved_versions).unwrap_or_default();
    let c_json = CString::new(json).unwrap_or_default();
    
    CResolutionResult {
        resolved_count: result.resolved_versions.len() as u32,
        resolved_json: c_json.into_raw(),
        strategy_used: match result.strategy_used {
            ResolutionStrategy::Eulerian => 0,
            ResolutionStrategy::Hamiltonian => 1,
            ResolutionStrategy::AStar => 2,
            ResolutionStrategy::Hybrid => 3,
        },
        iterations: result.iterations as u32,
        stress_impact: result.stress_impact,
        error_message: ptr::null(),
    }
}

fn to_c_string(s: &str) -> *const c_char {
    let c_str = CString::new(s).unwrap_or_default();
    c_str.into_raw()
}

fn to_c_string_ptr(s: &Option<String>) -> *const c_char {
    match s {
        Some(str) => to_c_string(str),
        None => ptr::null(),
    }
}

fn from_c_string_ptr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        unsafe {
            CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string())
        }
    }
}

fn free_c_semverx(semver: &CSemverX) {
    unsafe {
        if !semver.prerelease.is_null() {
            let _ = CString::from_raw(semver.prerelease as *mut c_char);
        }
        if !semver.build.is_null() {
            let _ = CString::from_raw(semver.build as *mut c_char);
        }
        if !semver.environment.is_null() {
            let _ = CString::from_raw(semver.environment as *mut c_char);
        }
        if !semver.classifier.is_null() {
            let _ = CString::from_raw(semver.classifier as *mut c_char);
        }
        if !semver.intent.is_null() {
            let _ = CString::from_raw(semver.intent as *mut c_char);
        }
        if !semver.sei_statement_contract.is_null() {
            let _ = CString::from_raw(semver.sei_statement_contract as *mut c_char);
        }
        if !semver.sei_expression_signature.is_null() {
            let _ = CString::from_raw(semver.sei_expression_signature as *mut c_char);
        }
        if !semver.sei_intent_hash.is_null() {
            let _ = CString::from_raw(semver.sei_intent_hash as *mut c_char);
        }
    }
}