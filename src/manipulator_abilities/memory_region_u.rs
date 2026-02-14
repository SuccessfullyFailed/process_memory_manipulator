#[cfg(test)]
mod tests {
	use crate::{ MemoryRegion, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };


	
	#[test]
	fn test_memory_regions() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let regions:Vec<MemoryRegion<u64>> = pmm.memory_regions().unwrap();
		assert!(regions.len() != 0, "Could not get memory regions.");
		let mut address:u64 = 0;
		for region in regions {
			assert_eq!(region.base_address(), address);
			address += region.size();
		}
	}

	#[test]
	fn test_memory_allocation() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let new_memory_address:u64 = pmm.allocate_memory(64).unwrap();
		assert_eq!(unsafe { *(new_memory_address as *const [u8; 64]) }, [0; 64]);
	}

	#[test]
	fn test_memory_allocation_after_specific_address() {
		const ACCEPTABLE_OFFSET:u64 = 0xF000;

		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let temp_var:u8 = 0;
		let temp_var_address:u64 = &temp_var as *const u8 as u64;
		let new_memory_address:u64 = pmm.allocate_memory_near(64, temp_var_address).unwrap();
		assert!((new_memory_address.max(temp_var_address) - new_memory_address.min(temp_var_address)) < ACCEPTABLE_OFFSET);
		assert_eq!(pmm.read_bytes(new_memory_address, 64).unwrap(), [0; 64]);
	}
}