#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate tokio;

use std::io::Error as IoError;
use std::time::SystemTime;

use log::{debug, error, info, Level, log_enabled};

use crate::audio_client::IAudioClientWrapper;
use crate::audio_sink::AudioSink;
use crate::capture_client::BufferStatus;
use crate::device::Device;
use crate::stream_format::{SampleFormat, StreamFormat};
use crate::writer::AudioWriter;
use crate::writer::hound_writer::HoundWriter;

mod asr;
mod audio_client;
mod audio_sink;
mod buffer;
mod capture_client;
mod com;
mod device;
mod device_enumerator;
mod stream_format;
mod utils;
mod writer;

const DEFAULT_TIMEOUT_SECS: u64 = 1000;

unsafe fn capture_output_stream(
    client: IAudioClientWrapper,
    stream_format: StreamFormat,
) {
    match stream_format.sample_format {
        SampleFormat::F32 => {
            let sink: Box<HoundWriter<f32>> = Box::new(HoundWriter::new(stream_format));
            client.record::<f32>().stream_to_sink(sink);
        }
        SampleFormat::I32 => {
            let sink: Box<HoundWriter<i32>> = Box::new(HoundWriter::new(stream_format));
            client.record::<i32>().stream_to_sink(sink);
        }
        SampleFormat::I16 => {
            let sink: Box<HoundWriter<i16>> = Box::new(HoundWriter::new(stream_format));
            client.record::<i16>().stream_to_sink(sink);
        }
        SampleFormat::U16 => unimplemented!(),
    };
}

fn create_client(device: Device) -> IAudioClientWrapper {
    unsafe {
        IAudioClientWrapper::new(&device, true)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let device = Device::new();
    info!("Device: {}", device.name);
    let client = create_client(device);
    let stream_format = client.get_format().unwrap();
    debug!("Stream Format; {:?}", stream_format);
    unsafe {
        capture_output_stream(client, stream_format);
    }
}
