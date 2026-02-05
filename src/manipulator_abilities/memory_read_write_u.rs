#[cfg(test)]
mod tests {
	use crate::{ ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name};



	const TEST_VALUE:f64 = 12345.6789;
	const MODIFIED_TEST_VALUE:f64 = 2468.97531;



	#[test]
	fn test_read_process_memory_from_address() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let test_variable:f64 = TEST_VALUE;
		let test_variable_value:f64 = pmm.read(&test_variable as *const f64 as u64).unwrap();
		assert_eq!(test_variable_value, TEST_VALUE);
	}

	#[test]
	fn test_read_process_memory_from_pointer() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let test_variable:f64 = TEST_VALUE;
		let dummy_variable:f64 = 0.0;
		let test_variable_wrapper:[&f64; 8] = [&dummy_variable, &dummy_variable, &dummy_variable, &dummy_variable, &dummy_variable, &test_variable, &dummy_variable, &dummy_variable];
		let test_variable_wrapper_wrapper:&[&f64; 8] = &test_variable_wrapper;
		let test_variable_value:f64 = pmm.read([&test_variable_wrapper_wrapper as *const &[&f64; 8] as u64, 8 * 5, 0]).unwrap();
		assert_eq!(test_variable_value, TEST_VALUE);
	}

	#[test]
	fn test_write_process_memory_address() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let test_variable:f64 = TEST_VALUE;
		pmm.write(&test_variable as *const f64 as u64, MODIFIED_TEST_VALUE).unwrap();
		assert_eq!(test_variable, MODIFIED_TEST_VALUE);
	}

	#[test]
	fn test_write_process_memory_from_pointer() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let test_variable:f64 = TEST_VALUE;
		let dummy_variable:f64 = 0.0;
		let test_variable_wrapper:[&f64; 8] = [&dummy_variable, &dummy_variable, &dummy_variable, &dummy_variable, &dummy_variable, &test_variable, &dummy_variable, &dummy_variable];
		let test_variable_wrapper_wrapper:&[&f64; 8] = &test_variable_wrapper;
		pmm.write([&test_variable_wrapper_wrapper as *const &[&f64; 8] as u64, 8 * 5, 0], MODIFIED_TEST_VALUE).unwrap();
		assert_eq!(test_variable, MODIFIED_TEST_VALUE);
	}
}