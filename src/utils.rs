use winapi::um::winnt::HRESULT;

use crate::IoError;

#[inline]
pub(crate) fn check_result(result: HRESULT) -> Result<(), IoError> {
    if result < 0 {
        Err(IoError::from_raw_os_error(result))
    } else {
        Ok(())
    }
}

#[macro_export]
macro_rules! win_api {
    ($target_type:ty, $target:expr, $method:ident $(, $opt:expr)*) => {
    unsafe {
        use std::ptr;
        use crate::utils::check_result;
        let mut target_var : *mut $target_type = ptr::null_mut();
        let hr_result = $target.$method(
             $($opt),*,
             &mut target_var as *mut *mut $target_type as *mut _,
        );
        check_result(hr_result).unwrap();
        target_var
    }
    }
}
