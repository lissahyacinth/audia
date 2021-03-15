use serde::Serialize;

use crate::buffer::ExtensibleBuffer;
use crate::stream_format;
use crate::stream_format::StreamFormat;

mod asr_connector;
pub(crate) mod hound_writer;

pub(crate) trait AudioWriter<T>
    where
        T: hound::Sample + stream_format::Sample + Serialize + Copy,
{
    fn new(format: StreamFormat) -> Self
        where
            Self: Sized;
    fn write(
        &mut self,
        data: &ExtensibleBuffer<T>,
        frames_available: usize,
    ) -> Result<(), anyhow::Error>;
    fn close(&mut self) -> Result<(), anyhow::Error>;
}
