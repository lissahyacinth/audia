use winapi::um::coml2api::STGM_READ;
use winapi::um::functiondiscoverykeys_devpkey::PKEY_Device_FriendlyName;
use winapi::um::mmdeviceapi::{eConsole, eRender, IMMDevice};
use winapi::um::propidl::PROPVARIANT;
use winapi::um::propsys::IPropertyStore;
use winapi::um::winnt::LPWSTR;

use crate::device_enumerator::ENUMERATOR;

pub(crate) struct Device {
    pub(crate) device: *mut IMMDevice,
    pub(crate) name: String,
}

impl Device {
    pub fn new() -> Self {
        use crate::utils::check_result;
        use std::ptr;
        let mut device: *mut IMMDevice = ptr::null_mut();
        unsafe {
            let h_result = ENUMERATOR
                .enumerator
                .as_ref()
                .unwrap()
                .GetDefaultAudioEndpoint(
                    eRender,
                    eConsole,
                    &mut device as *mut *mut IMMDevice as *mut _,
                );
            match check_result(h_result) {
                Err(e) => panic!(format!("[Create Device] - {x}", x = e)),
                Ok(_) => {
                    let name = get_device_name(device);
                    Device { device, name }
                }
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (*self.device).Release();
        }
    }
}

fn get_device_name(device: *mut IMMDevice) -> String {
    unsafe {
        use crate::utils::check_result;
        use std::ptr;
        let mut property_store: *mut IPropertyStore = ptr::null_mut();
        let h_result = device
            .as_ref()
            .unwrap()
            .OpenPropertyStore(STGM_READ, &mut property_store);
        match check_result(h_result) {
            Err(e) => panic!(format!("OpenPropertyStore - {x}", x = e)),
            Ok(_) => {
                let mut var_name: PROPVARIANT = { std::mem::zeroed() };
                let h_result = property_store
                    .as_ref()
                    .unwrap()
                    .GetValue(&PKEY_Device_FriendlyName, &mut var_name);
                match check_result(h_result) {
                    Err(e) => panic!(format!("GetValue - {x}", x = e)),
                    Ok(_) => {
                        let string_property = var_name.data.pwszVal();
                        let string_length: usize =
                            winapi::shared::stralign::uaw_wcslen(*string_property);
                        let device_name: String = String::from_utf16(std::slice::from_raw_parts(
                            *string_property,
                            string_length,
                        ))
                            .unwrap();
                        return device_name;
                    }
                }
            }
        }
    }
}
