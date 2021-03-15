use futures::StreamExt;

use crate::stream_format;
use crate::stream_format::{Sample, SampleFormat};

const BUFFER_SECS: usize = 2;
const BUFFER_LENGTH: usize = 192_000 * BUFFER_SECS;

pub(crate) struct Buffer {
    data: *mut (),
    len: usize,
    sample_format: SampleFormat,
}

#[derive(Debug)]
pub(crate) struct ExtensibleBuffer<T>
    where
        T: hound::Sample + stream_format::Sample,
{
    pub data: Vec<T>,
    pub len: usize,
    max_len: usize,
    sample_format: SampleFormat,
}

impl<T> ExtensibleBuffer<T>
    where
        T: hound::Sample + stream_format::Sample,
{
    fn new(data: Vec<T>, sample_format: SampleFormat) -> ExtensibleBuffer<T> {
        dbg!("New Buffer");
        let len: usize = data.len();
        ExtensibleBuffer {
            data,
            len,
            max_len: BUFFER_LENGTH,
            sample_format,
        }
    }

    pub(crate) unsafe fn from_raw_parts(
        data: *mut T,
        len: usize,
        sample_format: SampleFormat,
    ) -> Self {
        assert!(len <= BUFFER_LENGTH);
        ExtensibleBuffer {
            data: Vec::from_raw_parts(data, len, len),
            len,
            max_len: BUFFER_LENGTH,
            sample_format,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn len_unused_buffer(&self) -> usize {
        self.max_len - self.len()
    }

    fn has_unused_buffer(&self) -> bool {
        self.len_unused_buffer() > 0
    }

    /// Add elements to internal buffer
    ///
    /// Buffer is of a fixed vector size and uses rotation to maintain some of the previous
    /// elements in the stack.
    // FIXME: Issue with Extension Causing a Pointer Error.
    pub(crate) fn extend(&mut self, data: &[T], data_len: usize) {
        if !data.is_empty() {
            if !self.has_unused_buffer() {
                dbg!("No Unused Buffer");
                let start_index = self.max_len - data_len;
                self.data.rotate_left(data_len);
                dbg!("Rotated");
                dbg!(self.max_len);
                dbg!(start_index);
                dbg!(data.len());
                self.data.splice(start_index.., data.iter().cloned());
                dbg!("Splice");
            } else {
                dbg!("Has Unused Buffer");
                dbg!(self.len);
                dbg!(self.max_len);
                let usable_elements = std::cmp::min(data_len, self.len_unused_buffer());
                dbg!(usable_elements);
                if data_len <= usable_elements {
                    dbg!("Less than usable elements");

                    self.data.extend_from_slice(&data);
                    self.len = self.len + data_len;
                } else {
                    dbg!("More data than usable elements");
                    self.data.extend_from_slice(&data[..usable_elements]);
                    self.len = self.len + usable_elements;
                    self.extend(&data[usable_elements..], data_len - usable_elements);
                }
            }
        }
    }

    pub(crate) fn as_slice(&self) -> Option<&[T]> {
        if self.len > self.data.len() {
            None
        } else {
            Some(&self.data[0..self.len])
        }
    }
}

impl Buffer {
    pub(crate) unsafe fn from_raw_parts(
        data: *mut (),
        len: usize,
        sample_format: SampleFormat,
    ) -> Self {
        assert!(len <= BUFFER_LENGTH);
        Buffer {
            data,
            len: BUFFER_LENGTH,
            sample_format,
        }
    }

    /// Bytestream
    /// Number of Samples * Size of each Sample
    pub fn bytes(&self) -> &[u8] {
        let len = self.len * self.sample_format.sample_size();
        unsafe { std::slice::from_raw_parts(self.data as *const u8, len) }
    }

    /// Number of Samples
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice<T>(&self) -> Option<&[T]>
        where
            T: Sample,
    {
        if T::FORMAT == self.sample_format {
            unsafe { Some(std::slice::from_raw_parts(self.data as *const T, self.len)) }
        } else {
            dbg!(self.sample_format);
            dbg!(T::FORMAT);
            None
        }
    }
}
