#[cfg(test)]
mod tests {
	use crate::{ MemoryScanResult, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };
	use mini_rand::RandomNumber;


	
	#[test]
	fn test_memory_regions() {
		const MAX_ALLOWED_ATTEMPTS:usize = 32;
		const MAX_ALLOWED_RESULTS:usize = 4;

		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let mut hidden_value:f32 = f32::random();
		let hidden_value_address:u64 = &hidden_value as *const f32 as u64;
		let mut scan_results:MemoryScanResult<u64, f32> = pmm.scan_exact_value(hidden_value).unwrap();
		for _attempt_index in 0..MAX_ALLOWED_ATTEMPTS {
			hidden_value = f32::random();
			scan_results = pmm.re_scan_exact_value(hidden_value, &scan_results).unwrap();
			if scan_results.results().len() <= 1 {
				break;
			}
		}
		
		assert!(scan_results.results().len() < MAX_ALLOWED_RESULTS, "Max allowed results exceeded");
		assert!(scan_results.results().iter().any(|(address, value)| value == &hidden_value && address == &hidden_value_address), "Hidden value not found");
	}
}