mod process_memory_manipulator;
mod manipulator_abilities;
mod address_source;
mod access_token;
mod process_handle;
mod memory_data_type;
mod memory_data_type_u;

pub use process_memory_manipulator::*;
pub use manipulator_abilities::*;
pub use address_source::*;
pub use access_token::*;
pub use process_handle::*;
pub use memory_data_type::*;



/// Get the process name of this program.
#[cfg(test)]
pub(crate) fn active_process_name() -> String {
	use std::sync::{ Mutex, MutexGuard };

	static PROCESS_NAME_CACHE:Mutex<Option<String>> = Mutex::new(None);
	let mut process_name_cache_handle:MutexGuard<'_, Option<String>> = PROCESS_NAME_CACHE.lock().unwrap();
	match &*process_name_cache_handle {
		Some(process_name) => process_name.to_string(),
		None => {
			let exe_path:std::path::PathBuf = std::env::current_exe().unwrap();
			let exe_name:std::borrow::Cow<'_, str> = exe_path.file_name().unwrap().to_string_lossy();
			*process_name_cache_handle = Some(exe_name.to_string());
			process_name_cache_handle.as_ref().unwrap().to_string()
		}
	}
}