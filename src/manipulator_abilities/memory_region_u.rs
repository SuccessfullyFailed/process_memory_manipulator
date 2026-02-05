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
}