use std::{fs, io};
use std::marker::PhantomData;

use anyhow::Error;
use serde::Serialize;

use crate::buffer::ExtensibleBuffer;
use crate::stream_format;
use crate::stream_format::StreamFormat;
use crate::writer::AudioWriter;

pub(crate) struct HoundWriter<T> where
    T: hound::Sample + stream_format::Sample + Serialize + Copy, {
    format: StreamFormat,
    spec: hound::WavSpec,
    internal_writer: Option<hound::WavWriter<io::BufWriter<fs::File>>>,
    phantom_data: PhantomData<T>,
}

impl<T> AudioWriter<T> for HoundWriter<T> where
    T: hound::Sample + stream_format::Sample + Serialize + Copy {
    fn new(format: StreamFormat) -> Self {
        let spec = hound::WavSpec {
            channels: format.n_channels as u16,
            sample_rate: format.n_sample_per_sec,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        HoundWriter {
            format,
            spec,
            internal_writer: Some(hound::WavWriter::create("Example.wav", spec).unwrap()),
            phantom_data: PhantomData,
        }
    }

    fn write(&mut self, data: &ExtensibleBuffer<T>, frames_available: usize) -> Result<(), Error> {
        match self.internal_writer {
            Some(ref mut writer) => {
                match data.as_slice() {
                    Some(data_slice) => {
                        for x in data_slice {
                            writer.write_sample(*x);
                        }
                        Ok(())
                    }
                    None => panic!("No Slice Data available, but tried to write"),
                }
            },
            None => Err(anyhow!("Writer not initialised"))
        }
    }

    fn close(&mut self) -> Result<(), Error> {
        let writer = std::mem::take(&mut self.internal_writer);
        (writer.unwrap()).finalize().unwrap();
        Ok(())
    }
}
