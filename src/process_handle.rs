use winapi::{ ctypes::c_void, um::{ winnt::HANDLE as WinHandle, handleapi::CloseHandle, processthreadsapi::OpenProcess, tlhelp32::{ CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next, TH32CS_SNAPPROCESS } } };
use std::{ error::Error, ffi::CStr, mem };
use crate::MemoryAccessToken;



pub struct ProcessHandle {
	pub(crate) handle:WinHandle,
	pub(crate) access:MemoryAccessToken
}
impl ProcessHandle {

	/* CONSTRUCTOR METHODS */

	/// Create a new process handle.
	pub fn new<T:ProcessIdentifier>(process_identifier:T, access_token:MemoryAccessToken) -> Result<ProcessHandle, Box<dyn Error>> {
		unsafe {

			// Create a snapshot of the current processes list.
			let snapshot:*mut c_void = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
			if snapshot.is_null() {
				return Err("Could not create snapshot of process list.".into());
			}
		
			// Create a process-data buffer for winapi to write the entries from the list to.
			let mut entry:PROCESSENTRY32 = mem::zeroed();
			entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
		
			// Loop Through processes until the correct one is found.
			if Process32First(snapshot, &mut entry) == 0 {
				return Err("Could not find first process in process list snapshot.".into());
			}
			loop {
				if process_identifier.matches_entry(&entry) {

					// Open a handle to the newly found PID.
					let pid:u32 = entry.th32ProcessID;
					let handle:WinHandle = OpenProcess(access_token.0, 0, pid);
		
					// Return a process handle with the found data.
					return Ok(ProcessHandle {
						handle: handle,
						access: access_token
					});
				}

				// If no processes left, unable to find process.
				if Process32Next(snapshot, &mut entry) == 0 {
					break;
				}
			}

			Err(format!("Could not find process '{}'.", process_identifier.name()).into())
		}
	}
}
impl Drop for ProcessHandle {
	fn drop(&mut self) {
		unsafe { CloseHandle(self.handle); }
	}
}



pub trait ProcessIdentifier {
	fn name(&self) -> String;
	fn matches_entry(&self, entry:&PROCESSENTRY32) -> bool;
}
impl ProcessIdentifier for &dyn ProcessIdentifier {
	fn name(&self) -> String {
		(*self).name()
	}
	fn matches_entry(&self, entry:&PROCESSENTRY32) -> bool {
		(*self).matches_entry(entry)
	}
}
impl ProcessIdentifier for u32 {
	fn name(&self) -> String {
		self.to_string()
	}
	fn matches_entry(&self, entry:&PROCESSENTRY32) -> bool {
		entry.th32ProcessID == *self
	}
}
impl ProcessIdentifier for &str {
	fn name(&self) -> String {
		self.to_string()
	}
	fn matches_entry(&self, entry:&PROCESSENTRY32) -> bool {
		let name_bytes:&[u8] = unsafe { CStr::from_ptr(entry.szExeFile.as_ptr()).to_bytes() };
		if let Ok(entry_process_name) = str::from_utf8(name_bytes) {
			entry_process_name == *self
		} else {
			false
		}
	}
}