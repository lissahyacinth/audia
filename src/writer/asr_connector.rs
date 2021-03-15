use anyhow::Error;
use futures::executor::block_on;
use log::debug;
use serde::Serialize;

use crate::asr::python_net_request::{send_to_python, TorchPacket};
use crate::buffer::ExtensibleBuffer;
use crate::stream_format;
use crate::stream_format::StreamFormat;
use crate::writer::AudioWriter;

pub(crate) struct ASRConnector {
    format: StreamFormat,
}

impl<T> AudioWriter<T> for ASRConnector
    where
        T: hound::Sample + stream_format::Sample + Serialize + Copy,
{
    fn new(format: StreamFormat) -> Self {
        ASRConnector { format }
    }

    fn write(&mut self, data: &ExtensibleBuffer<T>, frames_available: usize) -> Result<(), Error> {
        match block_on(send_to_python(TorchPacket {
            data_packet: data.data.clone(),
            data_size: data.len,
            channels: self.format.n_channels as usize,
        })) {
            Ok(TextPrediction) => {
                debug!("Prediction: {:?}", TextPrediction);
                Ok(())
            },
            Err(e) => panic!("Failed to return prediction")
        }
    }

    fn close(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
