// src/lib.rs
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VersionState {
    Stable,
    Legacy,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemVerX {
    pub major: u32,
    pub major_state: VersionState,
    pub minor: u32,
    pub minor_state: VersionState,
    pub patch: u32,
    pub patch_state: VersionState,
    pub guid: Uuid,
    pub checksum: String,
}

impl SemVerX {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            major_state: VersionState::Stable,
            minor,
            minor_state: VersionState::Stable,
            patch,
            patch_state: VersionState::Stable,
            guid: Uuid::new_v4(),
            checksum: String::new(),
        }
    }

    pub fn with_states(
        major: (u32, VersionState),
        minor: (u32, VersionState),
        patch: (u32, VersionState),
    ) -> Self {
        Self {
            major: major.0,
            major_state: major.1,
            minor: minor.0,
            minor_state: minor.1,
            patch: patch.0,
            patch_state: patch.1,
            guid: Uuid::new_v4(),
            checksum: String::new(),
        }
    }
}

impl fmt::Display for SemVerX {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "v{}.{:?}.{}.{:?}.{}.{:?}",
            self.major, self.major_state,
            self.minor, self.minor_state,
            self.patch, self.patch_state
        )
    }
}

pub mod p2p;
pub mod resolver;
pub mod fault_tolerance;
