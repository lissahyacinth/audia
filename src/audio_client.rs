use std::mem::MaybeUninit;

use log::debug;
use serde::Serialize;
use winapi::shared::mmreg::WAVEFORMATEX;
use winapi::um::audioclient::{IAudioCaptureClient, IAudioClient, IID_IAudioCaptureClient};
use winapi::um::audioclient::{AUDCLNT_E_DEVICE_INVALIDATED, IID_IAudioClient};
use winapi::um::audiosessiontypes::{AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK};
use winapi::um::combaseapi::CLSCTX_ALL;
use winapi::um::mmdeviceapi::IMMDevice;

use crate::capture_client::RecordingAudioClient;
use crate::device::Device;
use crate::stream_format;
use crate::stream_format::StreamFormat;

const REFTIME_PER_SEC: i64 = 10_000_000;

pub(crate) struct UninitialisedAudioClientWrapper {
    pub(crate) iaudio_client: *mut IAudioClient,
}

pub(crate) struct IAudioClientWrapper {
    pub(crate) iaudio_client: *mut IAudioClient,
    format: Option<StreamFormat>,
    raw_format: Option<WAVEFORMATEX>,
}

impl IAudioClientWrapper {
    pub(crate) unsafe fn new(device: &Device, loopback: bool) -> Self {
        let client = UninitialisedAudioClientWrapper::new(&*(*device).device);
        debug!("Created Client");
        client.initialize(loopback)
    }

    pub(crate) fn buffer_duration(&self) -> Result<std::time::Duration, anyhow::Error> {
        if let Some(format) = &self.format {
            let res = self.get_buffer_size() as i64 / format.n_sample_per_sec as i64;
            Ok(::std::time::Duration::from_secs(res as u64))
        } else {
            Err(anyhow!("Format was not set for Audio Client"))
        }
    }

    pub(crate) fn get_buffer_size(&self) -> u32 {
        use crate::utils::check_result;
        let mut buffer_size = 0u32;
        unsafe {
            let hr = self
                .iaudio_client
                .as_ref()
                .unwrap()
                .GetBufferSize(&mut buffer_size);
            match check_result(hr) {
                Err(e) => panic!(format!("[Audio Client - Get Buffer Size] {x}", x = e)),
                Ok(_) => buffer_size,
            }
        }
    }

    pub(crate) fn get_raw_format(&self) -> Option<WAVEFORMATEX> {
        self.raw_format
    }

    pub(crate) fn get_format(&self) -> Option<StreamFormat> {
        self.format
    }

    pub(crate) fn record<T>(&self) -> RecordingAudioClient<T>
        where
            T: hound::Sample + stream_format::Sample + Serialize,
    {
        use crate::utils::check_result;
        use std::ptr;
        let mut capture_client: *mut IAudioCaptureClient = ptr::null_mut();

        unsafe {
            let hr_result = self.iaudio_client.as_ref().unwrap().Start();
            match check_result(hr_result) {
                Err(e) => panic!(format!("[Starting Recording] - {x}", x = e)),
                Ok(_) => {}
            }

            let hr_result = self.iaudio_client.as_ref().unwrap().GetService(
                &IID_IAudioCaptureClient,
                &mut capture_client as *mut *mut IAudioCaptureClient as *mut _,
            );
            match check_result(hr_result) {
                Err(ref e) if e.raw_os_error() == Some(AUDCLNT_E_DEVICE_INVALIDATED) => {
                    panic!("Audio Client Invalidated")
                }
                Err(e) => panic!(format!("[Recording Client - Record] - {x}", x = e)),
                Ok(_) => RecordingAudioClient {
                    audio_client: &self,
                    capture_client,
                    buffer: None,
                    format: self.format.unwrap(),
                },
            }
        }
    }
}

impl UninitialisedAudioClientWrapper {
    pub(crate) unsafe fn new(device: &IMMDevice) -> Self {
        use crate::utils::check_result;
        use std::ptr;
        let mut uninit_client_wrapper = MaybeUninit::uninit().assume_init();
        unsafe {
            let h_result = device.Activate(
                &IID_IAudioClient,
                CLSCTX_ALL,
                ptr::null_mut(),
                &mut uninit_client_wrapper,
            );
            match check_result(h_result) {
                Err(e) => panic!(format!("[New Device] - {x}", x = e)),
                Ok(_) => UninitialisedAudioClientWrapper {
                    iaudio_client: uninit_client_wrapper as *mut _,
                },
            }
        }
    }

    // TODO: Add loopback variable back
    pub(crate) fn initialize(self, _loopback: bool) -> IAudioClientWrapper {
        use std::ptr;
        let mut wrapper = IAudioClientWrapper {
            iaudio_client: self.iaudio_client,
            format: None,
            raw_format: None,
        };

        debug!("Created Wrapper");

        let mut mix_fmt: *mut WAVEFORMATEX = ptr::null_mut();

        unsafe {
            use crate::utils::check_result;
            let hr_result = wrapper
                .iaudio_client
                .as_ref()
                .unwrap()
                .GetMixFormat(&mut mix_fmt as *mut *mut WAVEFORMATEX as *mut _);

            debug!("Got Mix Format");

            match check_result(hr_result) {
                Err(e) => panic!(format!("[Audio GetMixFormat] - {x}", x = e)),
                Ok(_) => {
                    wrapper.raw_format = Some(*mix_fmt);
                    wrapper.format = Some(wrapper.raw_format.unwrap().into());
                    let hr_result = wrapper.iaudio_client.as_ref().unwrap().Initialize(
                        AUDCLNT_SHAREMODE_SHARED,
                        AUDCLNT_STREAMFLAGS_LOOPBACK,
                        REFTIME_PER_SEC,
                        0,
                        mix_fmt,
                        ptr::null(),
                    );
                    debug!("Initialized Audio Client");
                    match check_result(hr_result) {
                        Err(e) => panic!(format!("[Audio Initialize] - {x}", x = e)),
                        Ok(_) => wrapper,
                    }
                }
            }
        }
    }
}
