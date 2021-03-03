use thiserror::Error;
use uuid::Uuid;

type ErrorSource = Box<dyn std::error::Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid api http request")]
    InvalidApiRequest { source: ErrorSource },
    #[error("invalid api http status code")]
    InvalidApiStatusCode { code: u16 },
    #[error("invalid api status")]
    InvalidApiStatus,
    #[error("invalid api response")]
    InvalidApiResponse { source: ErrorSource },
    #[error("invalid tooltip for action {id} of type {name}")]
    InvalidTooltip {
        source: Option<ErrorSource>,
        id: Uuid,
        name: String,
    },
    #[error("invalid end date ({end})")]
    InvalidEndDate {
        source: ErrorSource,
        end: u64,
    },
}