#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod error;
mod open_ai_request;
mod open_ai_response;
mod soil_report;

use crate::error::{Error, InvalidResponseError, Result};
use crate::open_ai_request::{Content, ContentData, ImageUrl, Message, OpenAIRequest};
use crate::open_ai_response::OpenAIResponse;
pub use crate::soil_report::SoilReport;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use once_cell::sync::Lazy;
use pdfium_render::prelude::*;
use regex::Regex;
use reqwest::Client;
use serde_json::json;
use std::io::Cursor;

/// Regex pattern to capture the JSON content between triple backticks.
static JSON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"```json\s*([\s\S]+?)\s*```").unwrap());

/// Parses a PDF soil report, returning the data as a [`SoilReport`].
///
/// ## Arguments
/// - `pdf_file_path`: location of the soil report PDF file
/// - `pdfium_path`: location of the Pdfium shared library
/// - `open_ai_key`: OpenAI API key
pub async fn extract_soil_data(
    pdf_file_path: &str,
    pdfium_path: &str,
    open_ai_key: &str,
) -> Result<SoilReport> {
    // Convert PDF to PNG images in memory
    let png_images = convert_pdf_to_png_memory(pdf_file_path, pdfium_path)?;
    // Post images to OpenAI
    process_soil_report(png_images, open_ai_key).await
}

fn convert_pdf_to_png_memory(pdf_path: &str, pdfium_path: &str) -> Result<Vec<Vec<u8>>> {
    // Bind to a Pdfium library in the same directory as our Rust executable;
    // failing that, fall back to using a Pdfium library provided by the operating system.
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(pdfium_path))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );

    // Load the document from the given path
    let document = pdfium.load_pdf_from_file(pdf_path, None)?;

    // Set rendering options that will be applied to all pages
    let render_config = PdfRenderConfig::new()
        .set_target_width(1024)
        .set_maximum_height(1024);

    // Render each page to a bitmap image and save each image to a vector
    document
        .pages()
        .iter()
        .try_fold(Vec::new(), |mut png_images, page| {
            let image = page
                .render_with_config(&render_config)?
                .as_image() // Renders this page to an image::DynamicImage
                .into_rgb8(); // Converts it to an image::Image
            let mut png_data = Vec::new();
            // Write the image data to the Cursor
            image.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)?;
            png_images.push(png_data);
            Ok(png_images)
        })
}

fn get_first_message_content(response: &OpenAIResponse) -> Option<&String> {
    response
        .choices
        .first()
        .map(|choice| &choice.message.content)
}

fn extract_json(text: &str) -> Option<String> {
    // Search for the JSON block
    JSON_RE
        .captures(text)
        .and_then(|cap| cap.get(1).map(|json| json.as_str().to_owned()))
}

// OpenAI Docs for image processing: https://platform.openai.com/docs/guides/vision?lang=curl
async fn process_soil_report(images: Vec<Vec<u8>>, open_ai_key: &str) -> Result<SoilReport> {
    // Convert all images to Base64
    let base64images = images.into_iter().map(|img| BASE64_STANDARD.encode(img));

    // Build the OpenAI request
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/chat/completions";
    let prompt = include_str!("resources/ai_prompt.txt"); // Approximately 2 cents per prompt:  https://platform.openai.com/usage

    let mut request = OpenAIRequest {
        model: "gpt-4o-mini".to_owned(),
        messages: vec![Message {
            role: "user".to_owned(),
            content: vec![Content {
                r#type: "text".to_owned(),
                data: ContentData::Text {
                    text: prompt.to_owned(),
                },
            }],
        }],
        max_tokens: 1500,
    };

    // Attach all images to the request
    for base64image in base64images {
        let content_data = Content {
            r#type: "image_url".to_owned(),
            data: ContentData::ImageUrl {
                image_url: ImageUrl {
                    url: format!("data:image/png;base64,{}", base64image),
                    detail: "high".to_owned(),
                },
            },
        };

        request.messages[0].content.push(content_data);
    }

    let json_request = json!(request);

    // Send the request to OpenAI
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", open_ai_key))
        .json(&json_request)
        .send()
        .await?;

    if response.status().is_success() {
        let api_response = response.json::<OpenAIResponse>().await?;

        match get_first_message_content(&api_response) {
            Some(text) => match extract_json(text) {
                Some(json) => Ok(serde_json::from_str::<SoilReport>(&json)?),
                None => Err(Error::InvalidResponse(InvalidResponseError::NoJsonSection)),
            },
            None => Err(Error::InvalidResponse(InvalidResponseError::NoMessages)),
        }
    } else {
        let error_text = response.text().await?;
        Err(Error::OpenAIError(error_text))
    }
}
