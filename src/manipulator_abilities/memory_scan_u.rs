#[cfg(test)]
mod tests {
	use crate::{ MemoryDataType, MemoryScanResult, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };
	use mini_rand::RandomNumber;



	#[test]
	fn test_memory_scan() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let hidden_value:f32 = f32::random();
		let hidden_value_address:u64 = &hidden_value as *const f32 as u64;
		let scan_results:MemoryScanResult<u64, f32> = pmm.scan_exact_value(hidden_value).unwrap();
		
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().contains(&(hidden_value_address, hidden_value)), "Scan results do not contain hidden value.");
	}

	#[test]
	fn test_memory_re_scan() {
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
		
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().len() < MAX_ALLOWED_RESULTS, "Max allowed results exceeded.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results.");
	}

	#[test]
	fn test_memory_scan_endian_mismatch() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, true);

		let original_value:f32 = f32::random();
		let hidden_value:f32 = original_value.mdt_flip_endian();
		let hidden_value_address:u64 = &hidden_value as *const f32 as u64;
		let scan_results:MemoryScanResult<u64, f32> = pmm.scan_exact_value(original_value).unwrap();
		
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().contains(&(hidden_value_address, original_value)), "Scan results do not contain hidden value.");
	}

	#[test]
	#[allow(unused_assignments)]
	fn test_memory_re_scan_endian_mismatch() {
		const MAX_ALLOWED_ATTEMPTS:usize = 32;
		const MAX_ALLOWED_RESULTS:usize = 8;

		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, true);

		let mut original_value:f32 = f32::random();
		let mut hidden_value:f32 = original_value.mdt_flip_endian();
		let hidden_value_address:u64 = &hidden_value as *const f32 as u64;
		let mut scan_results:MemoryScanResult<u64, f32> = pmm.scan_exact_value(original_value).unwrap();
		println!("{}", scan_results.results().len());
		for _attempt_index in 0..MAX_ALLOWED_ATTEMPTS {
			original_value = f32::random();
			hidden_value = original_value.mdt_flip_endian();
			scan_results = pmm.re_scan_exact_value(original_value, &scan_results).unwrap();
			println!("{}", scan_results.results().len());
			if scan_results.results().len() <= MAX_ALLOWED_RESULTS {
				break;
			}
		}
		
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		//assert!(scan_results.results().len() < MAX_ALLOWED_RESULTS, "Max allowed results exceeded.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &original_value), "Hidden value not found in scan results.");
	}
}