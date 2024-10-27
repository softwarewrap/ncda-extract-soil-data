//! Library error types.

use image::ImageError;
use pdfium_render::prelude::*;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

/// Error type representing all invalid API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum InvalidResponseError {
    /// The response contains no JSON section.
    #[error("unable to locate the json section of the response")]
    NoJsonSection,
    /// The response contains no messages.
    #[error("the response contains no messages")]
    NoMessages,
}

/// Error type representing all possible library errors.
#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred while parsing a PDF document.
    #[error("PDF error: {0}")]
    PdfError(#[from] PdfiumError),
    /// An error occurred while reading or writing an image file.
    #[error("image error: {0}")]
    ImageError(#[from] ImageError),
    /// An error occurred while performing a web request.
    #[error("web request error: {0}")]
    WebRequestError(#[from] ReqwestError),
    /// An error was returned with the OpenAI response.
    #[error("OpenAI error: {0}")]
    OpenAIError(String),
    /// An error occurred while serializing or deserializing a JSON object.
    #[error("JSON error: {0}")]
    JsonError(#[from] SerdeJsonError),
    /// An invalid response was returned.
    #[error("invalid response: {0}")]
    InvalidResponse(InvalidResponseError),
}

/// Library result.
pub type Result<T> = core::result::Result<T, Error>;
