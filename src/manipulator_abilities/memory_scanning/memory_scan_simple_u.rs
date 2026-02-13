#[cfg(test)]
mod tests {
	use crate::{ MemoryDataType, MemoryScanResult, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };
	use mini_rand::RandomNumber;
	use std::i32;



	#[test]
	fn test_memory_scan() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let hidden_value:f32 = f32::random();
		let hidden_value_address:u64 = &hidden_value as *const f32 as u64;
		let scan_results:MemoryScanResult<u64, f32> = pmm.scan_value_exact(hidden_value).unwrap();
		
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
		let mut scan_results:MemoryScanResult<u64, f32> = pmm.scan_value_exact(hidden_value).unwrap();
		for _attempt_index in 0..MAX_ALLOWED_ATTEMPTS {
			hidden_value = f32::random();
			scan_results = pmm.re_scan_value_exact(hidden_value, &scan_results).unwrap();
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
		let scan_results:MemoryScanResult<u64, f32> = pmm.scan_value_exact(original_value).unwrap();
		
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
		let mut scan_results:MemoryScanResult<u64, f32> = pmm.scan_value_exact(original_value).unwrap();
		for _attempt_index in 0..MAX_ALLOWED_ATTEMPTS {
			original_value = f32::random();
			hidden_value = original_value.mdt_flip_endian();
			scan_results = pmm.re_scan_value_exact(original_value, &scan_results).unwrap();
			if scan_results.results().len() <= MAX_ALLOWED_RESULTS {
				break;
			}
		}
		
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().len() <= MAX_ALLOWED_RESULTS, "Max allowed results exceeded.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &original_value), "Hidden value not found in scan results.");
	}

	#[test]
	fn test_memory_simple_scan_types() {
		let mut hidden_value:i32 = i8::random() as i32;
		let hidden_value_address:u64 = &hidden_value as *const i32 as u64;

		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		println!("{}", hidden_value);
		let mut scan_results:MemoryScanResult<u64, i32> = pmm.scan_value_exact(hidden_value).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 'scan_value_exact'.");

		println!("{}", hidden_value);
		scan_results = pmm.scan_value_between(hidden_value - 1..hidden_value + 1).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 'scan_value_between'.");

		hidden_value = i32::MIN + 4;
		println!("{}", hidden_value);
		scan_results = pmm.scan_value_less_than(i32::MIN + 8).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 'scan_value_less_than'.");

		hidden_value = i32::MAX - 4;
		println!("{}", hidden_value);
		scan_results = pmm.scan_value_greater_than(i32::MAX - 8).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 'scan_value_greater_than'.");

		hidden_value = i8::random() as i32;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_between(hidden_value - 8..hidden_value + 16, &scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_between'.");

		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_unchanged(&scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_unchanged'.");

		hidden_value += i8::random() as i32;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_changed(&scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_changed'.");

		hidden_value += i8::random().abs() as i32;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_increased(&scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_increased'.");

		hidden_value -= i8::random().abs() as i32;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_decreased(&scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_decreased'.");

		let increase:i32 = i8::random().abs() as i32;
		hidden_value += increase;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_increased_by(increase, &scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_increased_by'.");

		let decrease:i32 = i8::random().abs() as i32;
		hidden_value -= decrease;
		println!("{}", hidden_value);
		scan_results = pmm.re_scan_value_decreased_by(decrease, &scan_results).unwrap();
		assert!(!scan_results.results().is_empty(), "No scan results found.");
		assert!(scan_results.results().iter().any(|(address, value)| address == &hidden_value_address && value == &hidden_value), "Hidden value not found in scan results after 're_scan_value_decreased_by'.");
	}
}