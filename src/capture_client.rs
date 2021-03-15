use core::mem;

use log::debug;
use serde::Serialize;
use winapi::shared::basetsd::UINT64;
use winapi::shared::minwindef::{BYTE, DWORD};
use winapi::um::audioclient::IAudioCaptureClient;

use crate::audio_client::IAudioClientWrapper;
use crate::audio_sink::AudioSink;
use crate::buffer::ExtensibleBuffer;
use crate::stream_format;
use crate::stream_format::{Sample, SampleFormat, StreamFormat};
use crate::utils::check_result;
use crate::writer::AudioWriter;

// REFERENCE_TIME time units per second and per millisecond
const REFTIME_PER_SEC: i64 = 10_000_000;
const REFTIME_PER_MILLISEC: i64 = 10_000;

pub(crate) enum BufferStatus {
    Streaming(*mut BYTE),
    NoData,
}

pub(crate) struct RecordingAudioClient<'a, T>
    where
        T: hound::Sample + stream_format::Sample + Serialize,
{
    pub(crate) audio_client: &'a IAudioClientWrapper,
    pub(crate) capture_client: *mut IAudioCaptureClient,
    pub(crate) buffer: Option<ExtensibleBuffer<T>>,
    pub(crate) format: StreamFormat,
}

impl<T> RecordingAudioClient<'_, T>
    where
        T: hound::Sample + stream_format::Sample + Serialize + std::fmt::Debug,
{
    /// Retrieve Hardware Audio Buffer, and copy into Internal Buffer
    pub(crate) fn get_buffer(&mut self, num_frames_available: &mut u32) -> BufferStatus {
        // TODO: Does `num_frames_available` need to be mutable?
        use std::ptr;
        // Flags do get set! Set to 231
        let mut _flags: *mut DWORD = ptr::null_mut();
        let mut buffer: *mut BYTE = ptr::null_mut();
        unsafe {
            let mut qpc_position: UINT64 = 0;
            let mut flags = mem::MaybeUninit::uninit();
            let h_result = (*self.capture_client).GetBuffer(
                &mut buffer,
                num_frames_available,
                flags.as_mut_ptr(),
                ptr::null_mut(),
                &mut qpc_position,
            );
            if *num_frames_available == 0 {
                return BufferStatus::NoData
            }
            match check_result(h_result) {
                Err(e) => panic!(format!("GetBuffer - {x}", x = e)),
                Ok(_) => BufferStatus::Streaming(buffer)
            }
        }
    }

    pub(crate) fn write_to_internal_vector(
        &mut self,
        buffer: *mut BYTE,
        num_frames_available: usize,
    ) -> Result<(), anyhow::Error> {
        debug_assert!(!buffer.is_null());
        let len = num_frames_available as usize * self.format.n_block_align as usize
            / self.format.sample_format.sample_size();

        match self.buffer {
            Some(ref mut internal_buffer) => unsafe {
                let mut data: Vec<T> = Vec::with_capacity(len);
                std::ptr::copy(buffer as *const T, data.as_mut_ptr(), len);
                (*internal_buffer).extend(&data, len);
            },
            None => unsafe {
                self.buffer = Some(ExtensibleBuffer::from_raw_parts(
                    buffer as *mut T,
                    len,
                    self.format.sample_format,
                ));
            },
        }
        Ok(())
    }

    /// Stream Audio Client Input to AudioSink
    pub(crate) fn stream_to_sink(&mut self, mut sink: Box<dyn AudioWriter<T>>) {
        let mut is_done = false;
        let mut num_frames_available = 0;
        let actual_duration = (REFTIME_PER_SEC / REFTIME_PER_MILLISEC) as f32
            * self.audio_client.get_buffer_size() as f32
            / self.format.n_sample_per_sec as f32;
        while !is_done {
            ::std::thread::sleep(std::time::Duration::from_millis(
                (actual_duration / 2.0) as u64,
            ));
            debug!("Sleep for {}ms", (actual_duration / 2.0) as u64);
            let mut packet_length = self.get_next_packet_size();
            while packet_length > 0 {
                match self.get_buffer(&mut num_frames_available) {
                    BufferStatus::Streaming(buffer) => {
                        debug!("Streaming - {} Frames Available", num_frames_available);
                        match self.write_to_internal_vector(
                            buffer,
                            num_frames_available as usize,
                        ) {
                            Ok(_) => {
                                (*sink).write(
                                    self.buffer.as_ref().unwrap(),
                                    num_frames_available as usize,
                                );
                            }
                            Err(e) => panic!("{}", e),
                        }
                    }
                    BufferStatus::NoData => {}
                }
                self.release_buffer(num_frames_available).unwrap();
                packet_length = self.get_next_packet_size();
            }
        }
    }

    /// Fetch Packet size from Audio Hardware
    pub(crate) fn get_next_packet_size(&self) -> u32 {
        let mut num_frames = 0;
        unsafe {
            let h_result = self
                .capture_client
                .as_ref()
                .unwrap()
                .GetNextPacketSize(&mut num_frames);
            match check_result(h_result) {
                Err(e) => panic!(format!("[Get Next Packet Size] - {x}", x = e)),
                Ok(_) => num_frames,
            }
        }
    }

    /// Clear Audio Hardware Buffer
    /// Audio Buffer is only available in C, and is therefore v.unsafe. The buffer must be
    /// cleared between reads in `GetBuffer`
    pub(crate) fn release_buffer(&mut self, n_frames: u32) -> Result<(), anyhow::Error> {
        match self.buffer {
            None => return Ok(()),
            Some(_) => unsafe {
                let h_result = self
                    .capture_client
                    .as_ref()
                    .unwrap()
                    .ReleaseBuffer(n_frames);
                match check_result(h_result) {
                    Err(e) => Err(anyhow!(format!("[Release Buffer Failed] {x}", x = e))),
                    Ok(_) => Ok(()),
                }
            },
        }
    }
}

impl<'a, T> Drop for RecordingAudioClient<'a, T>
    where
        T: hound::Sample + stream_format::Sample + Serialize,
{
    fn drop(&mut self) {
        unsafe {
            (*self.capture_client).Release();
        }
    }
}
