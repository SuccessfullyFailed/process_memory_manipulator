use winapi::{ um::{ errhandlingapi::GetLastError, memoryapi::{ VirtualAllocEx, VirtualQueryEx }, winnt::{ HANDLE as WinHandle, MEM_COMMIT, MEM_FREE, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY } } };
use crate::{ AddressSourceType, MemoryAccessToken, ProcessMemoryManipulator };
use std::{ error::Error, mem };



#[derive(Clone)]
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
	pub fn allocate_memory(&mut self, size:AddressType) -> Result<AddressType, Box<dyn Error>> where AddressType:PartialOrd {
		self.allocate_memory_at(size, AddressType::default())
	}

	/// Allocate new memory of the given size near the given address.
	pub fn allocate_memory_near(&mut self, size:AddressType, target_address:AddressType, max_allowed_offset:AddressType) -> Result<AddressType, Box<dyn Error>> where AddressType:PartialOrd {
		let min_address:AddressType = if target_address > max_allowed_offset { target_address - max_allowed_offset } else { AddressType::default() };
		let max_address:AddressType = target_address + max_allowed_offset;
		let mut address:AddressType = min_address;
		while let Ok(region) = self.memory_region_at(address) {
			if region.base_address >= min_address && region.state == MEM_FREE {
				if let Ok(allocated_address) = self.allocate_memory_at(size, region.base_address) {
					return Ok(allocated_address);
				}
			}
			address += region.size;
			if address > max_address {
				break;
			}
		}

		Err("Could not allocate memory within given bounds.".into())
	}

	/// Allocate new memory of the given size at the given address. Will try to allocate anywhere when the address is 0.
	pub fn allocate_memory_at(&mut self, size:AddressType, target_address:AddressType) -> Result<AddressType, Box<dyn Error>> where AddressType:PartialOrd {
		const ALLOCATE_ACCESS:MemoryAccessToken = MemoryAccessToken::PROCESS_VM_OPERATION;

		let remote_buffer:u64 = unsafe { VirtualAllocEx(self.win_handle(ALLOCATE_ACCESS)?, target_address.to_c_void_ptr_mut(), size.to_usize(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) as u64 };
		if remote_buffer == 0 {
		  	Err(format!("Failed to allocate memory in the remote process. Error code: '{}'.", unsafe { GetLastError() }).into())
		} else {
			Ok(AddressType::from_u64(remote_buffer))
		}
	}



	/// Get all memory regions.
	pub fn memory_regions(&mut self) -> Result<Vec<MemoryRegion<AddressType>>, Box<dyn Error>> {
		let mut address:AddressType = AddressType::default();
		let mut results:Vec<MemoryRegion<AddressType>> = Vec::new();
		while let Ok(region) = self.memory_region_at(address) {
			address += region.size;
			results.push(region);
		}
		Ok(results)
	}

	/// Get the memory region at a specific address.
	pub fn memory_region_at(&mut self, address:AddressType) -> Result<MemoryRegion<AddressType>, Box<dyn Error>> {
		const MEMORY_REGION_FETCH_ACCESS:MemoryAccessToken = MemoryAccessToken::PROCESS_QUERY_INFORMATION;
		
		unsafe {
			let process_handle:WinHandle = self.win_handle(MEMORY_REGION_FETCH_ACCESS)?;
			let mut memory_basic_information:MEMORY_BASIC_INFORMATION = mem::zeroed();
			let query_result:usize = VirtualQueryEx(process_handle, address.to_c_void_ptr(), &mut memory_basic_information, mem::size_of::<MEMORY_BASIC_INFORMATION>());
			if query_result == 0 {
		  		Err(format!("Failed to get memory region at address {:#x}. Error code: '{}'.", address, GetLastError()).into())
			} else {
				Ok(MemoryRegion {
					base_address: AddressType::from_u64(memory_basic_information.BaseAddress as u64),
					size: AddressType::from_u64(memory_basic_information.RegionSize as u64),
					state: memory_basic_information.State,
					protection: memory_basic_information.Protect
				})
			}
		}
	}
}