use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SoilReport {
    #[serde(rename = "ReportNumber")]
    report_number: String,

    #[serde(rename = "SampledDate")]
    sampled_date: String,

    #[serde(rename = "Samples")]
    samples: Vec<Sample>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sample {
    SampleId: String,
    LimeHistory: String,
    Crop1: String,
    Crop2: String,
    Crop1LimeRecommendations: String,
    Crop2LimeRecommendations: String,
    pH: f32,
    NpkFertilizerRecommendations: String,
    PhosphorusIndex: u32,
    PotassiumIndex: u32,
    AdditionalTestResults: AdditionalTestResults,
}

#[derive(Serialize, Deserialize, Debug)]
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
    s_i: Option<u32>
}