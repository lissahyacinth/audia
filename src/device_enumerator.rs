use std::ptr;

use winapi::Interface;
use winapi::um::combaseapi::*;
use winapi::um::mmdeviceapi::*;
use winapi::um::mmdeviceapi::IMMDeviceEnumerator;

use crate::com;
use crate::utils::check_result;

/// RAII object around `IMMDeviceEnumerator`.
pub(crate) struct Enumerator {
    pub(crate) enumerator: *mut IMMDeviceEnumerator,
}

impl Enumerator {
    fn new(enumerator: *mut IMMDeviceEnumerator) -> Self {
        Enumerator { enumerator }
    }
}

unsafe impl Send for Enumerator {}

unsafe impl Sync for Enumerator {}

impl Drop for Enumerator {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (*self.enumerator).Release();
        }
    }
}

lazy_static! {
    pub(crate) static ref ENUMERATOR: Enumerator = {
        // COM initialization is thread local, but we only need to have COM initialized in the
        // thread we create the objects in
        com::com_initialized();

        // building the devices enumerator object
        unsafe {
            let mut enumerator: *mut IMMDeviceEnumerator = ptr::null_mut();

            let hresult = CoCreateInstance(
                &CLSID_MMDeviceEnumerator,
                ptr::null_mut(),
                CLSCTX_ALL,
                &IMMDeviceEnumerator::uuidof(),
                &mut enumerator as *mut *mut IMMDeviceEnumerator as *mut _,
            );

            check_result(hresult).unwrap();
            Enumerator::new(enumerator)
        }
    };
}
