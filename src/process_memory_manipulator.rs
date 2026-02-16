use crate::{ AddressSourceType, MemoryAccessToken, ProcessHandle };
use winapi::um::{ winnt::HANDLE as WinHandle };
use std::error::Error;



pub type ProcessMemoryManipulator64 = ProcessMemoryManipulator<u64>;
pub type ProcessMemoryManipulator32 = ProcessMemoryManipulator<u32>;



pub struct ProcessMemoryManipulator<AddressType:AddressSourceType> {
	process_name:String,
	process_handle:Option<ProcessHandle>,
	big_endian:bool,
	thread_count:usize,

	_address_default:AddressType
}
impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new process memory manipulator.
	pub fn new(process_name:&str, big_endian:bool) -> ProcessMemoryManipulator<AddressType> {
		ProcessMemoryManipulator {
			process_name: process_name.to_string(),
			process_handle: None,
			big_endian,
			thread_count: 1,

			_address_default: AddressType::default()
		}
	}

	/// Return self with another amount of threads allowed.
	pub fn with_thread_count(mut self, thread_count:usize) -> Self {
		if thread_count == 0 {
			panic!("Scanner thread count cannot be 0.");
		}
		self.thread_count = thread_count;
		self
	}



	/* PROPERTY GETTER METHODS */

	/// Get the process name of the manipulator.
	pub fn process_name(&self) -> &str {
		&self.process_name
	}

	/// Wether or not this manipulator is big endian.
	pub fn big_endian(&self) -> bool {
		self.big_endian
	}

	/// The amount of threads the manipulator is allowed to use.
	pub fn scanner_thread_count(&self) -> usize {
		self.thread_count
	}



	/* HANDLE METHODS */

	/// Get the attached windows handle. Will create a new handle if the current one does not meet access criteria.
	pub(crate) fn win_handle(&mut self, required_access:MemoryAccessToken) -> Result<WinHandle, Box<dyn Error>> {
		Ok(self.handle(required_access)?.handle)
	}

	/// Get the attached process handle. Will create a new handle if the current one does not meet access criteria.
	fn handle(&mut self, required_access:MemoryAccessToken) -> Result<&ProcessHandle, Box<dyn Error>> {
		self.open_handle(required_access)?;
		Ok(self.process_handle.as_ref().unwrap())
	}

	/// Open a process handle. Will keep the current one if it meets access criteria.
	pub fn open_handle(&mut self, access_token:MemoryAccessToken) -> Result<(), Box<dyn Error>> {
		let current_access:MemoryAccessToken = self.process_handle.as_ref().map(|handle| handle.access).unwrap_or_default();
		if self.process_handle.is_none() || current_access & access_token != access_token {
			self.process_handle = Some(ProcessHandle::new(&*self.process_name, current_access | access_token)?);
		}
		Ok(())
	}

	/// Close the process handle.
	pub fn close_handle(&mut self) {
		self.process_handle = None;
	}
}