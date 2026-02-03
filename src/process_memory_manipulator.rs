use crate::{ MemoryAccessToken, MemoryDataType, ProcessHandle, ProcessIdentifier };
use std::{ error::Error, fmt::{ Debug, LowerHex }, ptr };
use winapi::{ctypes::c_void, um::{errhandlingapi::GetLastError, memoryapi::{ReadProcessMemory, WriteProcessMemory}, winnt::HANDLE as WinHandle}};



pub type ProcessMemoryManipulator64 = ProcessMemoryManipulator<u64>;
pub type ProcessMemoryManipulator32 = ProcessMemoryManipulator<u32>;



pub struct ProcessMemoryManipulator<AddressType:AddressSourceType> {
	process_identifier:Box<dyn ProcessIdentifier>,
	process_handle:Option<ProcessHandle>,
	big_endian:bool,

	_address_default:AddressType
}
impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {

	/* CONSTRUCTOR METHODS */

	/// Create a new process memory manipulator.
	pub fn new<T:ProcessIdentifier + 'static>(process_identifier:T, big_endian:bool) -> ProcessMemoryManipulator<AddressType> {
		ProcessMemoryManipulator {
			process_identifier: Box::new(process_identifier),
			process_handle: None,
			big_endian,

			_address_default: AddressType::default()
		}
	}



	/* HANDLE METHODS */

	/// Get the attached windows handle. Will create a new handle if the current one does not meet access criteria.
	fn win_handle(&mut self, required_access:MemoryAccessToken) -> Result<WinHandle, Box<dyn Error>> {
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
			self.process_handle = Some(ProcessHandle::new(&*self.process_identifier, current_access | access_token)?);
		}
		Ok(())
	}

	/// Close the process handle.
	pub fn close_handle(&mut self) {
		self.process_handle = None;
	}



	/* MEMORY READ AND WRITE METHODS */

	/// Read an array of bytes from memory.
	pub fn read_bytes(&mut self, address:AddressType, amount_of_bytes:usize) -> Result<Vec<u8>, Box<dyn Error>> {		
		const READ_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::READ_CONTROL.0 | MemoryAccessToken::PROCESS_VM_READ.0);

		// Create pointer to buffer to write to.
		let mut buffer:Vec<u8> = vec![0; amount_of_bytes];
		let inner_buffer:&mut [u8] = &mut buffer[..];
		let buffer_ptr:*mut c_void = inner_buffer.as_mut_ptr() as *mut c_void;
		let mut bytes_read:usize = 0;

		// Read the memory.
		let exit_status:i32 = unsafe { ReadProcessMemory(self.win_handle(READ_ACCESS)?, address.to_c_void_ptr(), buffer_ptr, amount_of_bytes, &mut bytes_read) };
		if exit_status == 0 {
			return Err(format!("Memory Read on address {:#08x} failed with error code {}.", address, unsafe { GetLastError() }).into());
		}

		// Create and return value.
		Ok(buffer)
	}

	/// Read a value from memory.
	pub fn read<DataType:MemoryDataType>(&mut self, address:AddressType) -> Result<DataType, Box<dyn Error>> {
		let read_bytes:Vec<u8> = self.read_bytes(address, DataType::BYTES_SIZE)?;
		Ok(
			if self.big_endian {
				DataType::mdt_from_be_bytes(read_bytes)
			} else {
				DataType::mdt_from_le_bytes(read_bytes)
			}
		)
	}

	/// Write an array of bytes to memory.
	pub fn write_bytes(&mut self, address:AddressType, bytes:&[u8]) -> Result<(), Box<dyn Error>> {
		const WRITE_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::PROCESS_QUERY_INFORMATION.0 | MemoryAccessToken::PROCESS_VM_WRITE.0 | MemoryAccessToken::PROCESS_VM_OPERATION.0);

		// Write the memory.
		let exit_status:i32 = unsafe { WriteProcessMemory(self.win_handle(WRITE_ACCESS)?, address.to_c_void_ptr_mut(), bytes.as_ptr() as *const c_void, bytes.len(), ptr::null_mut()) };
		if exit_status == 0 {
			return Err(format!("Memory Write on address {:#02x} failed with error code {}.", address, unsafe { GetLastError() }).into());
		}

		// Return success.
		Ok(())
	}

	/// Write a value to memory.
	pub fn write<DataType:MemoryDataType>(&mut self, address:AddressType, value:DataType) -> Result<(), Box<dyn Error>> {
		let value_as_bytes:Vec<u8> = if self.big_endian { value.mdt_to_be_bytes() } else { value.mdt_to_le_bytes() };
		self.write_bytes(address, &value_as_bytes)
	}
}



pub trait AddressSourceType:Debug + Default + LowerHex {
	fn to_c_void_ptr(&self) -> *const c_void;
	fn to_c_void_ptr_mut(&self) -> *mut c_void;
}
impl AddressSourceType for u64 {
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
}
impl AddressSourceType for u32 {
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
}