use std::{ error::Error, fmt::{ Debug, Display, LowerHex }, ops::{Add, AddAssign, Sub, SubAssign} };
use winapi::{ ctypes::c_void, um::{ winnt::HANDLE as WinHandle } };
use crate::{ MemoryAccessToken, ProcessHandle };



pub type ProcessMemoryManipulator64 = ProcessMemoryManipulator<u64>;
pub type ProcessMemoryManipulator32 = ProcessMemoryManipulator<u32>;



pub struct ProcessMemoryManipulator<AddressType:AddressSourceType> {
	process_name:String,
	process_handle:Option<ProcessHandle>,
	big_endian:bool,

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

			_address_default: AddressType::default()
		}
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



pub trait AddressSourceType:Debug + Display + Default + LowerHex + Copy + PartialEq + Add<Output=Self> + AddAssign + Sub<Output=Self> + SubAssign {
	fn to_usize(&self) -> usize;
	fn to_c_void_ptr(&self) -> *const c_void;
	fn to_c_void_ptr_mut(&self) -> *mut c_void;
	fn from_u64(address:u64) -> Self;
	fn from_usize(address:usize) -> Self;
	fn wrapping_sub(self, address:Self) -> Self;
	fn two_gb() -> Self;
}
impl AddressSourceType for u64 {
	fn to_usize(&self) -> usize {
		*self as usize
	}
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
	fn from_u64(address:u64) -> Self {
		address
	}
	fn from_usize(address:usize) -> Self {
		address as u64
	}
	fn wrapping_sub(self, address:Self) -> Self {
		u64::wrapping_sub(self, address)
	}
	fn two_gb() -> Self {
		const BYTES:u64 = 2 * 1024 * 1024 * 1024;
		BYTES
	}
}
impl AddressSourceType for u32 {
	fn to_usize(&self) -> usize {
		*self as usize
	}
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
	fn from_u64(address:u64) -> Self {
		address as u32
	}
	fn from_usize(address:usize) -> Self {
		address as u32
	}
	fn wrapping_sub(self, address:Self) -> Self {
		u32::wrapping_sub(self, address)
	}
	fn two_gb() -> Self {
		const BYTES:u32 = 2 * 1024 * 1024 * 1024;
		BYTES
	}
}