use core::mem;

use winapi::shared::guiddef::GUID;
use winapi::shared::ksmedia;
use winapi::shared::mmreg;
use winapi::shared::mmreg::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleFormat {
    I16,
    I32,
    U16,
    F32,
}

impl SampleFormat {
    #[inline]
    pub fn sample_size(&self) -> usize {
        match *self {
            SampleFormat::I16 => mem::size_of::<i16>(),
            SampleFormat::I32 => mem::size_of::<i32>(),
            SampleFormat::U16 => mem::size_of::<u16>(),
            SampleFormat::F32 => mem::size_of::<f32>(),
        }
    }
}

/// Trait for containers that contain PCM data.
pub unsafe trait Sample: Copy + Clone + Sized {
    /// The `SampleFormat` corresponding to this data type.
    const FORMAT: SampleFormat;

    /// Turns the sample into its equivalent as a floating-point.
    fn to_f32(&self) -> f32;
    /// Converts this sample into a standard i16 sample.
    fn to_i16(&self) -> i16;
    /// Converts into i32 sample.
    fn to_i32(&self) -> i32;
    /// Converts this sample into a standard u16 sample.
    fn to_u16(&self) -> u16;

    /// Converts any sample type to this one by calling `to_i16`, `to_u16` or `to_f32`.
    fn from<S>(_: &S) -> Self
        where
            S: Sample;
}

unsafe impl Sample for i32 {
    const FORMAT: SampleFormat = SampleFormat::I32;
    #[inline]
    fn to_f32(&self) -> f32 {
        self.to_i32().to_f32()
    }

    #[inline]
    fn to_i32(&self) -> i32 {
        *self
    }

    #[inline]
    fn to_i16(&self) -> i16 {
        if *self >= 32768 {
            (*self - 32768) as i16
        } else {
            (*self as i16) - 32767 - 1
        }
    }

    #[inline]
    fn to_u16(&self) -> u16 {
        self.to_i16().to_u16()
    }

    #[inline]
    fn from<S>(sample: &S) -> Self
        where
            S: Sample,
    {
        sample.to_i32()
    }
}

unsafe impl Sample for u16 {
    const FORMAT: SampleFormat = SampleFormat::U16;

    #[inline]
    fn to_f32(&self) -> f32 {
        self.to_i16().to_f32()
    }

    #[inline]
    fn to_i32(&self) -> i32 {
        self.to_i16().to_i32()
    }

    #[inline]
    fn to_i16(&self) -> i16 {
        if *self >= 32768 {
            (*self - 32768) as i16
        } else {
            (*self as i16) - 32767 - 1
        }
    }

    #[inline]
    fn to_u16(&self) -> u16 {
        *self
    }

    #[inline]
    fn from<S>(sample: &S) -> Self
        where
            S: Sample,
    {
        sample.to_u16()
    }
}

unsafe impl Sample for i16 {
    const FORMAT: SampleFormat = SampleFormat::I16;

    #[inline]
    fn to_f32(&self) -> f32 {
        if *self < 0 {
            *self as f32 / -(i16::MIN as f32)
        } else {
            *self as f32 / i16::MAX as f32
        }
    }

    #[inline]
    fn to_i32(&self) -> i32 {
        *self as i32
    }

    #[inline]
    fn to_i16(&self) -> i16 {
        *self
    }

    #[inline]
    fn to_u16(&self) -> u16 {
        if *self < 0 {
            (*self - ::std::i16::MIN) as u16
        } else {
            (*self as u16) + 32768
        }
    }

    #[inline]
    fn from<S>(sample: &S) -> Self
        where
            S: Sample,
    {
        sample.to_i16()
    }
}

unsafe impl Sample for f32 {
    const FORMAT: SampleFormat = SampleFormat::F32;

    #[inline]
    fn to_f32(&self) -> f32 {
        *self
    }

    #[inline]
    fn to_i32(&self) -> i32 {
        if *self >= 0.0 {
            (*self * i32::MAX as f32) as i32
        } else {
            (-*self * i32::MIN as f32) as i32
        }
    }

    #[inline]
    fn to_i16(&self) -> i16 {
        if *self >= 0.0 {
            (*self * i16::MAX as f32) as i16
        } else {
            (-*self * i16::MIN as f32) as i16
        }
    }

    #[inline]
    fn to_u16(&self) -> u16 {
        (((*self + 1.0) * 0.5) * u16::MAX as f32).round() as u16
    }

    #[inline]
    fn from<S>(sample: &S) -> Self
        where
            S: Sample,
    {
        sample.to_f32()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FormatTag {
    PCM,
    IeeFloat,
    DRM,
    Extensible,
    Alaw,
    Mulaw,
    ADPCM,
    MPEG,
    DolbySpdif,
    WmaSpdif,
}

impl FormatTag {
    fn value(&self) -> u16 {
        match *self {
            FormatTag::PCM => WAVE_FORMAT_PCM,
            FormatTag::IeeFloat => WAVE_FORMAT_IEEE_FLOAT,
            FormatTag::DRM => WAVE_FORMAT_DRM,
            FormatTag::Extensible => WAVE_FORMAT_EXTENSIBLE,
            FormatTag::Alaw => WAVE_FORMAT_ALAW,
            FormatTag::Mulaw => WAVE_FORMAT_MULAW,
            FormatTag::ADPCM => WAVE_FORMAT_ADPCM,
            FormatTag::MPEG => WAVE_FORMAT_MPEG,
            FormatTag::DolbySpdif => WAVE_FORMAT_DOLBY_AC3_SPDIF,
            FormatTag::WmaSpdif => WAVE_FORMAT_WMASPDIF,
        }
    }
}

impl From<u16> for FormatTag {
    fn from(input: u16) -> FormatTag {
        if input == FormatTag::PCM.value() {
            FormatTag::PCM
        } else if input == FormatTag::IeeFloat.value() {
            FormatTag::IeeFloat
        } else if input == FormatTag::DRM.value() {
            FormatTag::DRM
        } else if input == FormatTag::Extensible.value() {
            FormatTag::Extensible
        } else if input == FormatTag::Alaw.value() {
            FormatTag::Alaw
        } else if input == FormatTag::Mulaw.value() {
            FormatTag::Mulaw
        } else if input == FormatTag::ADPCM.value() {
            FormatTag::ADPCM
        } else if input == FormatTag::MPEG.value() {
            FormatTag::MPEG
        } else if input == FormatTag::DolbySpdif.value() {
            FormatTag::DolbySpdif
        } else if input == FormatTag::WmaSpdif.value() {
            FormatTag::WmaSpdif
        } else {
            panic!("")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct StreamFormat {
    format_tag: FormatTag,
    pub(crate) n_channels: u32,
    pub(crate) n_sample_per_sec: u32,
    pub(crate) n_avg_bytes_per_sec: u32,
    pub(crate) n_block_align: u32,
    pub(crate) w_bits_per_sample: u32,
    cb_size: u32,
    pub(crate) sample_format: SampleFormat,
}

impl From<WAVEFORMATEX> for StreamFormat {
    fn from(format: WAVEFORMATEX) -> Self {
        unsafe {
            fn cmp_guid(a: &GUID, b: &GUID) -> bool {
                a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
            }
            let format_tag: FormatTag = format.wFormatTag.into();
            let sample_format = match (format.wBitsPerSample, format_tag) {
                (16, FormatTag::PCM) => SampleFormat::I16,
                (32, FormatTag::IeeFloat) => SampleFormat::F32,
                (32, _) => SampleFormat::F32,
                _ => panic!("Couldn't ascertain format"),
            };
            StreamFormat {
                format_tag,
                n_channels: format.nChannels.into(),
                n_sample_per_sec: format.nSamplesPerSec.into(),
                n_avg_bytes_per_sec: format.nAvgBytesPerSec.into(),
                n_block_align: format.nBlockAlign.into(),
                w_bits_per_sample: format.wBitsPerSample.into(),
                cb_size: format.cbSize.into(),
                sample_format,
            }
        }
    }
}
