use thiserror::Error;
use uuid::Uuid;

type ErrorSource = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot build json data")]
    Json { source: ErrorSource },
    #[error("invalid http request")]
    InvalidRequest { source: ErrorSource },
    #[error("invalid tooltip data")]
    InvalidTooltip {
        source: Option<ErrorSource>,
        id: Uuid,
        name: String,
    },
}