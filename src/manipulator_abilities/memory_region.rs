use winapi::{ ctypes::c_void, um::{ errhandlingapi::GetLastError, memoryapi::{ VirtualAllocEx, VirtualQueryEx }, winnt::{ HANDLE as WinHandle, MEM_COMMIT, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY } } };
use crate::{ AddressSourceType, MemoryAccessToken, ProcessMemoryManipulator };
use std::{ error::Error, mem };



pub struct MemoryRegion<AddressType:AddressSourceType> {
	base_address:AddressType,
	size:AddressType,
	state:u32,
	protection:u32
}
impl<AddressType:AddressSourceType> MemoryRegion<AddressType> {

	/* PROPERTY GETTER METHODS */

	/// Get the base address of the memory region.
	pub fn base_address(&self) -> AddressType {
		self.base_address
	}

	/// Get the size of the memory region in bytes.
	pub fn size(&self) -> AddressType {
		self.size
	}

	/// Get the state of the memory region.
	pub fn state(&self) -> u32 {
		self.state
	}

	/// Get the protection of the memory region.
	pub fn protection(&self) -> u32 {
		self.protection
	}

	/// Whether or not the region is readable.
	pub(crate) fn is_readable(&self) -> bool {
		self.state() == MEM_COMMIT &&
		[
			PAGE_READONLY,
			PAGE_READWRITE,
			PAGE_WRITECOPY,
			PAGE_EXECUTE_READ,
			PAGE_EXECUTE_READWRITE,
			PAGE_EXECUTE_WRITECOPY
		].iter().any(|token| self.protection & *token == *token)
	}
}



impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {

	/// Allocate new memory of the given size.
	pub fn allocate_memory(&mut self, size:AddressType) -> Result<AddressType, Box<dyn Error>> {
		self.allocate_memory_after(size, AddressType::default())
	}

	/// Allocate new memory of the given size after the given address.
	pub fn allocate_memory_after(&mut self, size:AddressType, start_address:AddressType) -> Result<AddressType, Box<dyn Error>> {
		const ALLOCATE_ACCESS:MemoryAccessToken = MemoryAccessToken::PROCESS_VM_OPERATION;

		let remote_buffer:u64 = unsafe { VirtualAllocEx(self.win_handle(ALLOCATE_ACCESS)?, start_address.to_c_void_ptr_mut(), size.to_usize(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as u64 };
		if remote_buffer == 0 {
		  	Err(format!("Failed to allocate memory in the remote process. Error code: '{}'.", unsafe { GetLastError() }).into())
		} else {
			Ok(AddressType::from_u64(remote_buffer))
		}
	}

	/// Get all memory regions.
	pub fn memory_regions(&mut self) -> Result<Vec<MemoryRegion<AddressType>>, Box<dyn Error>> {	
		const MEMORY_REGION_FETCH_ACCESS:MemoryAccessToken = MemoryAccessToken::PROCESS_QUERY_INFORMATION;

		unsafe {
			let process_handle:WinHandle = self.win_handle(MEMORY_REGION_FETCH_ACCESS)?;
			let mut address:usize = 0;
			let mut results:Vec<MemoryRegion<AddressType>> = Vec::new();
			let mut mbi:MEMORY_BASIC_INFORMATION = mem::zeroed();
			while VirtualQueryEx(process_handle, address as *const c_void, &mut mbi, mem::size_of::<MEMORY_BASIC_INFORMATION>()) != 0 {
				results.push(MemoryRegion {
					base_address: AddressType::from_u64(mbi.BaseAddress as u64),
					size: AddressType::from_u64(mbi.RegionSize as u64),
					state: mbi.State,
					protection: mbi.Protect
				});
				address += mbi.RegionSize;
			}
			Ok(results)
		}
	}
}