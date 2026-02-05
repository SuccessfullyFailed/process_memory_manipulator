#[cfg(test)]
mod tests {
	use crate::{ ModuleInfo, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };



	#[test]
	fn test_get_process_base_module() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let base_module:ModuleInfo<u64> = pmm.get_base_module_info().unwrap();
		assert_eq!(base_module.name(), &process_name);
		assert!(base_module.base_address() != 0, "Could not get base module base address.");
		assert!(base_module.entry_point() != 0, "Could not get base module entry point.");
		assert!(base_module.size() != 0, "Could not get base module size.");
	}
}