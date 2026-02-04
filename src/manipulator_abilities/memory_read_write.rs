use winapi::{ ctypes::c_void, um::{ errhandlingapi::GetLastError, memoryapi::{ ReadProcessMemory, WriteProcessMemory } } };
use crate::{ AddressSourceType, MemoryAccessToken, MemoryDataType, ProcessMemoryManipulator };
use std::{ ptr, error::Error };



impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {

	/// Read an array of bytes from memory.
	pub fn read_bytes<AddressReference:MemoryAddressReference<AddressType>>(&mut self, address_reference:AddressReference, amount_of_bytes:usize) -> Result<Vec<u8>, Box<dyn Error>> {		
		const READ_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::READ_CONTROL.0 | MemoryAccessToken::PROCESS_VM_READ.0);

		// Create pointer to buffer to write to.
		let mut buffer:Vec<u8> = vec![0; amount_of_bytes];
		let inner_buffer:&mut [u8] = &mut buffer[..];
		let buffer_ptr:*mut c_void = inner_buffer.as_mut_ptr() as *mut c_void;
		let mut bytes_read:usize = 0;

		// Read the memory.
		let address:AddressType = address_reference.to_raw_address(self)?;
		let exit_status:i32 = unsafe { ReadProcessMemory(self.win_handle(READ_ACCESS)?, address.to_c_void_ptr(), buffer_ptr, amount_of_bytes, &mut bytes_read) };
		if exit_status == 0 {
			return Err(format!("Memory Read on address {:#08x} failed with error code {}.", address, unsafe { GetLastError() }).into());
		}

		// Create and return value.
		Ok(buffer)
	}

	/// Read a value from memory.
	pub fn read<DataType:MemoryDataType, AddressReference:MemoryAddressReference<AddressType>>(&mut self, address_reference:AddressReference) -> Result<DataType, Box<dyn Error>> {
		let address:AddressType = address_reference.to_raw_address(self)?;
		let read_bytes:Vec<u8> = self.read_bytes(address, DataType::BYTES_SIZE)?;
		Ok(
			if self.big_endian() {
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
		let value_as_bytes:Vec<u8> = if self.big_endian() { value.mdt_to_be_bytes() } else { value.mdt_to_le_bytes() };
		self.write_bytes(address, &value_as_bytes)
	}
}



pub trait MemoryAddressReference<AddressType:AddressSourceType> {
	fn to_raw_address(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<AddressType, Box<dyn Error>>;
}
impl<AddressType:AddressSourceType + Clone> MemoryAddressReference<AddressType> for AddressType {
	fn to_raw_address(&self, _pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<AddressType, Box<dyn Error>> {
		Ok(self.clone())
	}
}
impl<AddressType:AddressSourceType + MemoryDataType> MemoryAddressReference<AddressType> for Vec<AddressType> {
	fn to_raw_address(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<AddressType, Box<dyn Error>> {
		self[..].to_raw_address(pmm)
	}
}
impl<AddressType:AddressSourceType + MemoryDataType, const ARRAY_SIZE:usize> MemoryAddressReference<AddressType> for [AddressType; ARRAY_SIZE] {
	fn to_raw_address(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<AddressType, Box<dyn Error>> {
		self[..].to_raw_address(pmm)
	}
}
impl<AddressType:AddressSourceType + MemoryDataType> MemoryAddressReference<AddressType> for [AddressType] {
	fn to_raw_address(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<AddressType, Box<dyn Error>> {
		if self.is_empty() {
			return Err("Could not get memory address from empty list. At least one address is required.".into());
		}
		let mut address:AddressType = self[0].clone();
		for offset in &self[1..] {
			address = pmm.read(address)?;
			address = address + offset.clone();
		}
		Ok(address)
	}
}