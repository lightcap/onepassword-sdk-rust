use std::path::PathBuf;

use libloading::{Library, Symbol};

use crate::core::Core;
use crate::errors::SdkError;

type InitClientFn = unsafe extern "C" fn(
    config_ptr: *const u8,
    config_len: usize,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
) -> i32;

type InvokeFn = unsafe extern "C" fn(
    invoke_ptr: *const u8,
    invoke_len: usize,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
) -> i32;

type ReleaseClientFn = unsafe extern "C" fn(client_id_ptr: *const u8, client_id_len: usize);

type FreeResponseFn = unsafe extern "C" fn(buf: *mut u8, len: usize);

pub(crate) struct SharedLibCore {
    _library: Library,
    init_client_fn: InitClientFn,
    invoke_fn: InvokeFn,
    release_client_fn: ReleaseClientFn,
    free_response_fn: FreeResponseFn,
}

// SAFETY: The shared library functions are thread-safe per 1Password's documentation.
unsafe impl Send for SharedLibCore {}
unsafe impl Sync for SharedLibCore {}

impl SharedLibCore {
    pub fn new(_account_name: &str) -> Result<Self, SdkError> {
        let lib_path = find_1password_lib_path()?;

        let library = unsafe {
            Library::new(&lib_path)
                .map_err(|e| SdkError::SharedLib(format!("failed to load library: {e}")))?
        };

        let init_client_fn: InitClientFn = unsafe {
            let sym: Symbol<InitClientFn> = library
                .get(b"init_client")
                .map_err(|e| SdkError::SharedLib(format!("symbol init_client not found: {e}")))?;
            *sym
        };

        let invoke_fn: InvokeFn = unsafe {
            let sym: Symbol<InvokeFn> = library
                .get(b"invoke")
                .map_err(|e| SdkError::SharedLib(format!("symbol invoke not found: {e}")))?;
            *sym
        };

        let release_client_fn: ReleaseClientFn = unsafe {
            let sym: Symbol<ReleaseClientFn> = library.get(b"release_client").map_err(|e| {
                SdkError::SharedLib(format!("symbol release_client not found: {e}"))
            })?;
            *sym
        };

        let free_response_fn: FreeResponseFn = unsafe {
            let sym: Symbol<FreeResponseFn> = library
                .get(b"free_response")
                .map_err(|e| SdkError::SharedLib(format!("symbol free_response not found: {e}")))?;
            *sym
        };

        Ok(Self {
            _library: library,
            init_client_fn,
            invoke_fn,
            release_client_fn,
            free_response_fn,
        })
    }

    fn call_and_read(
        &self,
        call: impl FnOnce(*mut *mut u8, *mut usize) -> i32,
    ) -> Result<Vec<u8>, SdkError> {
        let mut out_buf: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;

        let ret = call(&mut out_buf, &mut out_len);

        if ret != 0 {
            return Err(SdkError::SharedLib(format!(
                "shared library call failed with code {ret}"
            )));
        }

        if out_buf.is_null() || out_len == 0 {
            return Ok(Vec::new());
        }

        let result = unsafe { std::slice::from_raw_parts(out_buf, out_len).to_vec() };
        unsafe { (self.free_response_fn)(out_buf, out_len) };

        Ok(result)
    }
}

impl Core for SharedLibCore {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError> {
        let init_fn = self.init_client_fn;
        self.call_and_read(|out_buf, out_len| unsafe {
            init_fn(config.as_ptr(), config.len(), out_buf, out_len)
        })
    }

    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        let invoke = self.invoke_fn;
        self.call_and_read(|out_buf, out_len| unsafe {
            invoke(
                invoke_config.as_ptr(),
                invoke_config.len(),
                out_buf,
                out_len,
            )
        })
    }

    fn release_client(&self, client_id: &[u8]) {
        unsafe {
            (self.release_client_fn)(client_id.as_ptr(), client_id.len());
        }
    }
}

fn find_1password_lib_path() -> Result<PathBuf, SdkError> {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_default();

    let lib_name = if cfg!(target_os = "macos") {
        "libop_sdk_ipc_client.dylib"
    } else if cfg!(target_os = "windows") {
        "op_sdk_ipc_client.dll"
    } else {
        "libop_sdk_ipc_client.so"
    };

    let locations: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![
            PathBuf::from(format!(
                "/Applications/1Password.app/Contents/Frameworks/{lib_name}"
            )),
            home.join(format!(
                "Applications/1Password.app/Contents/Frameworks/{lib_name}"
            )),
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            PathBuf::from(format!("/usr/bin/1password/{lib_name}")),
            PathBuf::from(format!("/opt/1Password/{lib_name}")),
            PathBuf::from(format!("/snap/bin/1password/{lib_name}")),
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            home.join(format!(r"AppData\Local\1Password\{lib_name}")),
            PathBuf::from(format!(r"C:\Program Files\1Password\app\8\{lib_name}")),
            PathBuf::from(format!(
                r"C:\Program Files (x86)\1Password\app\8\{lib_name}"
            )),
        ]
    } else {
        return Err(SdkError::SharedLib(format!(
            "unsupported OS: {}",
            std::env::consts::OS
        )));
    };

    for path in &locations {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(SdkError::SharedLib(format!(
        "1Password desktop application not found. Searched: {}",
        locations
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}
