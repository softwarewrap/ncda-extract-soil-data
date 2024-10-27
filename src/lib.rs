extern crate reqwest;

use std::io::Cursor;

use anyhow::{anyhow, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use pdfium_render::prelude::*;
use regex::Regex;
use reqwest::Client;
use serde_json::json;
use tokio::runtime::Runtime;

use open_ai_request::{Content, ContentData, ImageUrl, Message, OpenAIRequest};
use open_ai_response::OpenAIResponse;
use soil_report::SoilReport;

mod open_ai_response;
mod open_ai_request;
mod soil_report;

// Utility function to convert a PDF soil report into a SoilReport data structure
// Arguments:
// pdf_file_path : location of the soil report PDF file
// pdfium_path: location of the Pdfium shared library
// open_ai_key: OpenAI API key
pub fn extract_soil_data(pdf_file_path: &str, pdfium_path: &str, open_ai_key: &str) -> Result<SoilReport> {
    println!("Converting PDF file to PNG images...");

    // Convert PDF to PNG images in memory
    let result = convert_pdf_to_png_memory(pdf_file_path, pdfium_path);
    match result {
        Ok(png_images) => {
            // Post images to OpenAI
            println!("Submitting {} pages to OpenAI", png_images.len());

            let result = process_soil_report(png_images, open_ai_key);
            match result {
                Ok(result) => Ok(result),
                Err(err) => Err(err)
            }
        }
        Err(err) => Err(err)
    }
}

fn convert_pdf_to_png_memory(pdf_path: &str, pdfium_path: &str) -> Result<Vec<Vec<u8>>> {
    // Bind to a Pdfium library in the same directory as our Rust executable;
    // failing that, fall back to using a Pdfium library provided by the operating system.
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(pdfium_path))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| anyhow!("Failed to load Pdfium library: {}", e))?,
    );

    // Load the document from the given path
    let document = pdfium.load_pdf_from_file(pdf_path, None)
        .map_err(|e| anyhow!("Failed to load PDF document: {}", e))?;

    // Set rendering options that will be applied to all pages
    let render_config = PdfRenderConfig::new()
        .set_target_width(1024)
        .set_maximum_height(1024);

    // Render each page to a bitmap image and save each image to a vector
    let mut png_images: Vec<Vec<u8>> = Vec::new();
    for (index, page) in document.pages().iter().enumerate() {
        let image = page.render_with_config(&render_config)
            .map_err(|e| anyhow!("Failed to render page {}: {}", index, e))?
            .as_image()  // Renders this page to an image::DynamicImage
            .into_rgb8(); // Converts it to an image::Image
        let mut png_data = Vec::new();
        // Write the image data to the Cursor
        image.write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)?;

        png_images.push(png_data);
    }

    Ok(png_images)
}

fn get_first_message_content(response: &OpenAIResponse) -> Option<&String> {
    response.choices.get(0).map(|choice| &choice.message.content)
}

fn extract_json(text: &str) -> Option<String> {
    // Define a regex pattern to capture the JSON content between triple backticks
    let re = Regex::new(r"```json\s*([\s\S]+?)\s*```").unwrap();

    // Search for the JSON block
    re.captures(text).and_then(|cap| cap.get(1).map(|json| json.as_str().to_string()))
}

// OpenAI Docs for image processing: https://platform.openai.com/docs/guides/vision?lang=curl
fn process_soil_report(images: Vec<Vec<u8>>, open_ai_key: &str) -> Result<SoilReport> {

    // Convert all images to Base64
    let mut base64images = Vec::new();
    for img in images {
        base64images.push(BASE64_STANDARD.encode(&img));
    }

    // Build the OpenAI request
    let client = Client::new();
    let api_url = "https://api.openai.com/v1/chat/completions";
    let prompt = include_str!("resources/ai_prompt.txt");  // Approximately 2 cents per prompt:  https://platform.openai.com/usage

    let mut request = OpenAIRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: vec![
                    Content {
                        r#type: "text".to_string(),
                        data: ContentData::Text {
                            text: prompt.to_string(),
                        },
                    }
                ],
            },
        ],
        max_tokens: 1500,
    };

    // Attach all images to the request
    for base64image in base64images {
        let content_data = Content {
            r#type: "image_url".to_string(),
            data: ContentData::ImageUrl {
                image_url: ImageUrl {
                    url: format!("data:image/png;base64,{}", base64image),
                    detail: "high".to_string(),
                },

            },
        };

        request.messages[0].content.push(content_data);
    }

    let json_request = json!(request);

    // Create a new Tokio runtime
    let rt = Runtime::new().unwrap();

    // Use the Tokio runtime to block on the async function
    rt.block_on(async {

        // Send the request to OpenAI
        let response = client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", open_ai_key))
            .json(&json_request)
            .send()
            .await?;

        if response.status().is_success() {
            let api_response: OpenAIResponse = response.json().await?;
            match get_first_message_content(&api_response) {
                Some(text) => match extract_json(text) {
                    Some(json) => {
                        // println!("Json = {}", json);  // Display the raw JSON
                        let report: SoilReport = serde_json::from_str(&json).unwrap();
                        Ok(report)
                    }
                    None => Err(anyhow!("Unable to location the json section of the response"))
                }

                None => Err(anyhow!("Invalid json response"))
            }
        } else {
            let error_text = response.text().await?;
            Err(anyhow!(error_text))
        }
    })
}
