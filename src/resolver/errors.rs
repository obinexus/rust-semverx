// resolver/errors.rs - Error type definitions

use std::fmt;
use crate::BubblingError;

#[derive(Debug, Clone)]
pub enum ResolutionError {
    ComponentNotFound(String),
    VersionConflict(String),
    CyclicDependency(String),
    MaxIterationsExceeded,
    GraphValidationFailed(String),
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionError::ComponentNotFound(id) => {
                write!(f, "Component not found: {}", id)
            }
            ResolutionError::VersionConflict(msg) => {
                write!(f, "Version conflict: {}", msg)
            }
            ResolutionError::CyclicDependency(msg) => {
                write!(f, "Cyclic dependency detected: {}", msg)
            }
            ResolutionError::MaxIterationsExceeded => {
                write!(f, "Maximum resolution iterations exceeded")
            }
            ResolutionError::GraphValidationFailed(msg) => {
                write!(f, "Graph validation failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for ResolutionError {}

// Conversion from ResolutionError to BubblingError
impl From<ResolutionError> for BubblingError {
    fn from(err: ResolutionError) -> Self {
        BubblingError::new(
            match &err {
                ResolutionError::ComponentNotFound(_) => 404,
                ResolutionError::VersionConflict(_) => 409,
                ResolutionError::CyclicDependency(_) => 508,
                ResolutionError::MaxIterationsExceeded => 503,
                ResolutionError::GraphValidationFailed(_) => 422,
            },
            err.to_string(),
        )
    }
}
