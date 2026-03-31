use std::path::PathBuf;

use base64::Engine;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use crate::core::Core;
use crate::errors::SdkError;

/// FFI signature for `op_sdk_ipc_send_message`.
///
/// int32_t op_sdk_ipc_send_message(
///     const uint8_t* msg_ptr, size_t msg_len,
///     uint8_t** out_buf, size_t* out_len, size_t* out_cap
/// );
type SendMessageFn = unsafe extern "C" fn(
    msg_ptr: *const u8,
    msg_len: usize,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
    out_cap: *mut usize,
) -> i32;

/// FFI signature for `op_sdk_ipc_free_response`.
///
/// void op_sdk_ipc_free_response(uint8_t* buf, size_t len, size_t cap);
type FreeResponseFn = unsafe extern "C" fn(buf: *mut u8, len: usize, cap: usize);

/// JSON request envelope sent to the shared library.
/// The Go SDK sends payload as base64 (Go's json.Marshal of []byte).
/// The shared library expects this format.
#[derive(Serialize)]
struct Request {
    kind: String,
    account_name: String,
    payload: String,
}

/// JSON response envelope received from the shared library.
/// The `payload` field is a byte array (matching Go's `[]byte` JSON encoding).
#[derive(Deserialize)]
struct Response {
    success: bool,
    payload: Vec<u8>,
}

pub(crate) struct SharedLibCore {
    account_name: String,
    _library: Library,
    send_message: SendMessageFn,
    free_response: FreeResponseFn,
}

// SAFETY: The shared library functions are thread-safe per 1Password's Go SDK usage
// (the Go SDK calls these across goroutines).
unsafe impl Send for SharedLibCore {}
unsafe impl Sync for SharedLibCore {}

impl SharedLibCore {
    pub fn new(account_name: &str) -> Result<Self, SdkError> {
        let lib_path = find_1password_lib_path()?;

        let library = unsafe {
            Library::new(&lib_path)
                .map_err(|e| SdkError::SharedLib(format!("failed to load library: {e}")))?
        };

        let send_message: SendMessageFn = unsafe {
            let sym: Symbol<SendMessageFn> = library
                .get(b"op_sdk_ipc_send_message")
                .map_err(|e| SdkError::SharedLib(format!("symbol not found: {e}")))?;
            *sym
        };

        let free_response: FreeResponseFn = unsafe {
            let sym: Symbol<FreeResponseFn> = library
                .get(b"op_sdk_ipc_free_response")
                .map_err(|e| SdkError::SharedLib(format!("symbol not found: {e}")))?;
            *sym
        };

        Ok(Self {
            account_name: account_name.to_string(),
            _library: library,
            send_message,
            free_response,
        })
    }

    fn call_shared_library(&self, kind: &str, payload: &[u8]) -> Result<Vec<u8>, SdkError> {
        let request = Request {
            kind: kind.to_string(),
            account_name: self.account_name.clone(),
            payload: base64::engine::general_purpose::STANDARD.encode(payload),
        };
        let input = serde_json::to_vec(&request)
            .map_err(|e| SdkError::SharedLib(format!("failed to serialize request: {e}")))?;

        let mut out_buf: *mut u8 = std::ptr::null_mut();
        let mut out_len: usize = 0;
        let mut out_cap: usize = 0;

        let ret_code = unsafe {
            (self.send_message)(
                input.as_ptr(),
                input.len(),
                &mut out_buf,
                &mut out_len,
                &mut out_cap,
            )
        };

        error_from_return_code(ret_code)?;

        // Copy the response bytes before freeing the library-owned buffer.
        // Always free if out_buf is non-null, even if out_len is 0.
        let resp_bytes = if !out_buf.is_null() {
            let bytes = if out_len > 0 {
                unsafe { std::slice::from_raw_parts(out_buf, out_len).to_vec() }
            } else {
                Vec::new()
            };
            unsafe { (self.free_response)(out_buf, out_len, out_cap) };
            bytes
        } else {
            Vec::new()
        };

        let response: Response = serde_json::from_slice(&resp_bytes)
            .map_err(|e| SdkError::SharedLib(format!("failed to parse response: {e}")))?;

        if response.success {
            Ok(response.payload)
        } else {
            let msg = String::from_utf8_lossy(&response.payload);
            Err(SdkError::SharedLib(msg.into_owned()))
        }
    }
}

impl Core for SharedLibCore {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError> {
        self.call_shared_library("init_client", config)
    }

    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        self.call_shared_library("invoke", invoke_config)
    }

    fn release_client(&self, client_id: &[u8]) {
        let _ = self.call_shared_library("release_client", client_id);
    }
}

fn error_from_return_code(ret_code: i32) -> Result<(), SdkError> {
    if ret_code == 0 {
        return Ok(());
    }

    let msg = if cfg!(target_os = "macos") {
        match ret_code {
            -3 => "desktop app connection channel is closed. Make sure Settings > Developer > Integrate with other apps is enabled".to_string(),
            -7 => "connection was unexpectedly dropped by the desktop app. Make sure the desktop app is running".to_string(),
            _ => format!("an internal error occurred. Return code: {ret_code}"),
        }
    } else {
        match ret_code {
            -2 => "desktop app connection channel is closed. Make sure Settings > Developer > Integrate with other apps is enabled".to_string(),
            -5 => "connection was unexpectedly dropped by the desktop app. Make sure the desktop app is running".to_string(),
            _ => format!("an internal error occurred. Return code: {ret_code}"),
        }
    };

    Err(SdkError::SharedLib(msg))
}

fn find_1password_lib_path() -> Result<PathBuf, SdkError> {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_default();

    let locations: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![
            PathBuf::from(
                "/Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib",
            ),
            home.join("Applications/1Password.app/Contents/Frameworks/libop_sdk_ipc_client.dylib"),
        ]
    } else if cfg!(target_os = "linux") {
        vec![
            PathBuf::from("/usr/bin/1password/libop_sdk_ipc_client.so"),
            PathBuf::from("/opt/1Password/libop_sdk_ipc_client.so"),
            PathBuf::from("/snap/bin/1password/libop_sdk_ipc_client.so"),
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            home.join(r"AppData\Local\1Password\op_sdk_ipc_client.dll"),
            PathBuf::from(r"C:\Program Files\1Password\app\8\op_sdk_ipc_client.dll"),
            PathBuf::from(r"C:\Program Files (x86)\1Password\app\8\op_sdk_ipc_client.dll"),
            home.join(r"AppData\Local\1Password\app\8\op_sdk_ipc_client.dll"),
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
