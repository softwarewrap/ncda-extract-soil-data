use serde::{Deserialize, Serialize};

/// A parsed soil report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoilReport {
    #[serde(rename = "ReportNumber")]
    report_number: String,

    #[serde(rename = "SampledDate")]
    sampled_date: String,

    #[serde(rename = "Samples")]
    samples: Vec<Sample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    #[serde(rename = "SampleId")]
    sample_id: String,

    #[serde(rename = "LimeHistory")]
    lime_history: String,

    #[serde(rename = "Crop1")]
    crop1: String,

    #[serde(rename = "Crop2")]
    crop2: String,

    #[serde(rename = "Crop1LimeRecommendations")]
    crop1_lime_recommendations: String,

    #[serde(rename = "Crop2LimeRecommendations")]
    crop2_lime_recommendations: String,

    #[serde(rename = "pH")]
    ph: f32,

    #[serde(rename = "NpkFertilizerRecommendations")]
    npk_fertilizer_recommendations: String,

    #[serde(rename = "PhosphorusIndex")]
    phosphorus_index: u32,

    #[serde(rename = "PotassiumIndex")]
    potassium_index: u32,

    #[serde(rename = "AdditionalTestResults")]
    additional_test_results: AdditionalTestResults,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdditionalTestResults {
    #[serde(rename = "HmPercent")]
    hm_percent: Option<f32>,

    #[serde(rename = "WV")]
    wv: Option<f32>,

    #[serde(rename = "CEC")]
    cec: Option<f32>,

    #[serde(rename = "Mn-I")]
    mn_i: Option<u32>,

    #[serde(rename = "Zn-I")]
    zn_i: Option<u32>,

    #[serde(rename = "Cu-I")]
    cu_i: Option<u32>,

    #[serde(rename = "S-I")]
    s_i: Option<u32>,
}
