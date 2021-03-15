use std::{fs, io};

use anyhow::Error;
use futures::executor::block_on;
use hound;
use serde::Serialize;

use crate::asr::python_net_request::{send_to_python, TorchPacket};
use crate::buffer::ExtensibleBuffer;
use crate::stream_format;
use crate::stream_format::StreamFormat;

pub(crate) struct AudioSink {
    format: Option<StreamFormat>,
    hound_spec: Option<hound::WavSpec>,
    hound_writer: Option<hound::WavWriter<io::BufWriter<fs::File>>>,
}

impl AudioSink {
    pub(crate) fn new() -> Self {
        AudioSink {
            format: None,
            hound_spec: None,
            hound_writer: None,
        }
    }

    fn send_to_python_model<T>(&mut self, data: &ExtensibleBuffer<T>)
        where
            T: hound::Sample + stream_format::Sample + Serialize,
    {
        let data = data.as_slice().unwrap().to_vec();
        let data_len = data.len();
        match block_on(send_to_python(TorchPacket {
            data_packet: data,
            data_size: data_len / 2,
            channels: 2,
        })) {
            Ok(response) => {
                dbg!(response.text);
            }
            Err(e) => panic!(format!("{x}", x = e)),
        }
    }
    /// Copy a Specified Number of Audio Frames from a Specified Buffer Location
    /// Record Audio Stream uses this function to read/save audio data from the shared buffer.
    ///
    /// While the audio sink requires data, CopyData outputs false through its third parameter.
    ///
    /// # Arguments
    /// * `p_data` Optional Raw Data from `GetBuffer`
    /// * `num_frames_available` - Frame Count from `GetBuffer`

    /// Set the Format for Copy Data to use for the data
    pub(crate) fn set_format(&mut self, format: StreamFormat) {
        self.format = Some(format);
    }
}
