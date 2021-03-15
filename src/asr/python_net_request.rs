use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct TorchPacket<T> {
    pub(crate) data_packet: Vec<T>,
    pub(crate) data_size: usize,
    pub(crate) channels: usize,
}

#[derive(Deserialize, Debug)]
pub(crate) struct TextPredictions {
    pub(crate) text: String,
}

lazy_static! {
    pub(crate) static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub(crate) async fn send_to_python<T>(
    input: TorchPacket<T>,
) -> Result<TextPredictions, Box<dyn Error>>
    where
        T: Serialize,
{
    let resp = CLIENT
        .post("http://127.0.0.1:8000/uncompressed")
        .timeout(std::time::Duration::from_millis(500))
        .json(&input)
        .send()
        .await?
        .json::<TextPredictions>()
        .await?;
    Ok(resp)
}
