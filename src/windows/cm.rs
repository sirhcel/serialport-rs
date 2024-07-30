use std::ffi::CStr;
use winapi::shared::minwindef::{MAX_PATH, ULONG};
use winapi::um::cfgmgr32::{CM_Get_Device_IDA, CM_Get_Parent, CONFIGRET, CR_SUCCESS, DEVINST};

// TODO: As DEVINST is only a type alias - what about a newtype for trusty device instance handles.
// struct DeviceInstance {
//     inner: DEVINST,
// }

pub(super) struct ParentInstances {
    instance: Option<DEVINST>,
}

impl ParentInstances {
    pub(super) unsafe fn from_handle(handle: DEVINST) -> Self {
        ParentInstances {
            instance: Some(handle),
        }
    }
}

impl Iterator for ParentInstances {
    type Item = DEVINST;

    fn next(&mut self) -> Option<Self::Item> {
        self.instance = self.instance.and_then(|instance| {
            let mut parent = 0;
            let status = unsafe { CM_Get_Parent(&mut parent, instance, 0) };
            if status == CR_SUCCESS {
                Some(parent)
            } else {
                // The documentation for CM_Get_Parent at only explicityl the successful outcome
                // CR_SUCCESS. It does not explicitly mention the case when we reached the root
                // device/the in so more parent and so we can't distinguish between this one and an
                // actual error.
                //
                // Let's consider everything but CR_SUCCESS as end of this iteration.
                //
                // See
                // https://learn.microsoft.com/en-us/windows/win32/api/cfgmgr32/nf-cfgmgr32-cm_get_parent.
                None
            }
        });

        self.instance
    }
}

pub(super) unsafe fn device_id(instance: DEVINST) -> std::result::Result<String, CONFIGRET> {
    let mut result_buf = [0i8; MAX_PATH];

    let res = unsafe {
        CM_Get_Device_IDA(
            instance,
            result_buf.as_mut_ptr(),
            (result_buf.len() - 1) as ULONG,
            0,
        )
    };

    if res == CR_SUCCESS {
        let end_of_buffer = result_buf.len() - 1;
        result_buf[end_of_buffer] = 0;
        Ok(unsafe {
            CStr::from_ptr(result_buf.as_ptr())
                .to_string_lossy()
                .into_owned()
        })
    } else {
        Err(res)
    }
}
