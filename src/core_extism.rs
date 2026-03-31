use std::sync::Mutex;

use extism::{Manifest, Plugin, PluginBuilder, UserData, ValType, Wasm};

use crate::core::{Core, MESSAGE_LIMIT};
use crate::errors::SdkError;

static CORE_WASM: &[u8] = include_bytes!("../wasm/core.wasm");

/// Allowed 1Password hosts that the WASM core may connect to.
const ALLOWED_HOSTS: &[&str] = &[
    "*.1password.com",
    "*.1password.ca",
    "*.1password.eu",
    "*.b5staging.com",
    "*.b5dev.com",
    "*.b5dev.ca",
    "*.b5dev.eu",
    "*.b5test.com",
    "*.b5test.ca",
    "*.b5test.eu",
    "*.b5rev.com",
    "*.b5local.com",
];

/// ExtismCore implements the [`Core`] trait using an embedded WASM binary
/// loaded through the Extism runtime.
pub(crate) struct ExtismCore {
    plugin: Mutex<Plugin>,
}

impl ExtismCore {
    /// Create a new `ExtismCore` by loading the embedded WASM binary and
    /// registering required host functions.
    pub fn new() -> Result<Self, SdkError> {
        let wasm = Wasm::data(CORE_WASM);
        let manifest =
            Manifest::new([wasm]).with_allowed_hosts(ALLOWED_HOSTS.iter().map(|h| h.to_string()));

        let no_data: UserData<()> = UserData::new(());

        let plugin = PluginBuilder::new(manifest)
            .with_wasi(true)
            // random_fill_imported: fills a buffer with crypto-random bytes
            // Takes I32 (length), returns I64 (pointer to allocated memory)
            .with_function_in_namespace(
                "op-extism-core",
                "random_fill_imported",
                [ValType::I32],
                [ValType::I64],
                no_data.clone(),
                random_fill,
            )
            // unix_time_milliseconds_imported in "op-now" namespace
            .with_function_in_namespace(
                "op-now",
                "unix_time_milliseconds_imported",
                [],
                [ValType::I64],
                no_data.clone(),
                unix_time_milliseconds,
            )
            // unix_time_milliseconds_imported in "zxcvbn" namespace
            .with_function_in_namespace(
                "zxcvbn",
                "unix_time_milliseconds_imported",
                [],
                [ValType::I64],
                no_data.clone(),
                unix_time_milliseconds,
            )
            // utc_offset_seconds in "op-time" namespace
            .with_function_in_namespace(
                "op-time",
                "utc_offset_seconds",
                [],
                [ValType::I64],
                no_data,
                utc_offset_seconds,
            )
            .build()
            .map_err(|e| SdkError::Plugin(format!("failed to initialize WASM plugin: {e}")))?;

        Ok(Self {
            plugin: Mutex::new(plugin),
        })
    }
}

impl Core for ExtismCore {
    fn init_client(&self, config: &[u8]) -> Result<Vec<u8>, SdkError> {
        let mut plugin = self.plugin.lock().unwrap();
        let res = plugin
            .call::<&[u8], &[u8]>("init_client", config)
            .map_err(|e| SdkError::Plugin(e.to_string()))?;
        Ok(res.to_vec())
    }

    fn invoke(&self, invoke_config: &[u8]) -> Result<Vec<u8>, SdkError> {
        if invoke_config.len() > MESSAGE_LIMIT {
            return Err(SdkError::Config(format!(
                "message size exceeds the limit of {} bytes",
                MESSAGE_LIMIT
            )));
        }
        let mut plugin = self.plugin.lock().unwrap();
        let res = plugin
            .call::<&[u8], &[u8]>("invoke", invoke_config)
            .map_err(|e| SdkError::Plugin(e.to_string()))?;
        Ok(res.to_vec())
    }

    fn release_client(&self, client_id: &[u8]) {
        let mut plugin = self.plugin.lock().unwrap();
        let _ = plugin.call::<&[u8], &[u8]>("release_client", client_id);
    }
}

/// Host function: generate cryptographically random bytes.
///
/// The WASM core calls this with a length (I32) and expects back a pointer (I64)
/// to the allocated memory block containing the random bytes.
fn random_fill(
    plugin: &mut extism::CurrentPlugin,
    inputs: &[extism::Val],
    outputs: &mut [extism::Val],
    _user_data: UserData<()>,
) -> Result<(), extism::Error> {
    let length = inputs[0].unwrap_i32() as u32;
    let mut buf = vec![0u8; length as usize];
    getrandom::getrandom(&mut buf)
        .map_err(|e| extism::Error::msg(format!("getrandom failed: {e}")))?;
    let handle = plugin.memory_new(&buf)?;
    outputs[0] = plugin.memory_to_val(handle);
    Ok(())
}

/// Host function: return current Unix timestamp in milliseconds.
fn unix_time_milliseconds(
    _plugin: &mut extism::CurrentPlugin,
    _inputs: &[extism::Val],
    outputs: &mut [extism::Val],
    _user_data: UserData<()>,
) -> Result<(), extism::Error> {
    let now = chrono::Utc::now().timestamp_millis();
    outputs[0] = extism::Val::I64(now);
    Ok(())
}

/// Host function: return the local timezone's UTC offset in seconds.
fn utc_offset_seconds(
    _plugin: &mut extism::CurrentPlugin,
    _inputs: &[extism::Val],
    outputs: &mut [extism::Val],
    _user_data: UserData<()>,
) -> Result<(), extism::Error> {
    let offset_secs = chrono::Local::now().offset().local_minus_utc();
    outputs[0] = extism::Val::I64(offset_secs as i64);
    Ok(())
}
