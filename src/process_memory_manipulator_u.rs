#[cfg(test)]
mod test {
	use std::{ thread::sleep, time::Duration, process::{ Child, Command, Stdio } };
	use crate::ProcessMemoryManipulator64;
	
	const TEST_PROGRAM_EXE_NAME:&str = "process_memory_manipulator_test_program.exe";
	const TEST_PROGRAM_PATH:&str = "_unit_test_program/target/debug/process_memory_manipulator_test_program.exe";
	const TEST_PROGRAM_COMPILE_PATH:&str = "_unit_test_program";
	const AWAIT_ADDRESS_PRINT_INTERVAL:Duration = Duration::from_millis(1);
	const AWAIT_ADDRESS_PRINT_ATTEMPTS:usize = 200;

	/// test all functionality in one test to make sure the unit_test_program only has to be ran once.
	#[test]
	fn process_memory_manipulator_full_test() {

		// Run the unit test program.
		let mut test_program:Child = create_test_program_process();
		let addresses:Vec<u64> = fetch_printed_addresses(&mut test_program);
		println!("Passed addresses from unit test tool: {:?}", addresses.iter().map(|address| format!("{:#02x}", address)).collect::<Vec<String>>());

		// Run all tests.
		read_process_memory(addresses[0]);
		write_process_memory(addresses[1], addresses[2], addresses[3]);
		read_process_memory_from_pointer(addresses[4], addresses[5], addresses[6]);

		// Kill program.
		test_program.kill().expect("Could not kill the test program.");
	}



	/* HELPER METHODS */

	/// Create the child process.
	fn create_test_program_process() -> Child {

		// Compile.
		let compile_status_code:i32 = Command::new("cargo").arg("build").current_dir(TEST_PROGRAM_COMPILE_PATH).output().expect("Could not compile unit test program").status.code().unwrap();
		if compile_status_code != 0 {
			panic!("Unit test program compiled with exit status {compile_status_code}.");
		}

		// Run.
		Command::new(TEST_PROGRAM_PATH).stdout(Stdio::piped()).spawn().expect("Could not run test program")
	}

	/// Get a list of required addresses from the output of the test program.
	fn fetch_printed_addresses(test_program_process:&mut Child) -> Vec<u64> {
		use std::{ process::ChildStdout, io::{ BufRead, BufReader, Lines } };
		const END_TAG:u64 = 0x666;

		let mut addresses:Vec<u64> = Vec::new();
		for _ in 0..AWAIT_ADDRESS_PRINT_ATTEMPTS {
			if let Some(stdout) = test_program_process.stdout.take() {				
				let mut reader:Lines<BufReader<ChildStdout>> = BufReader::new(stdout).lines();
				while let Some(line) = reader.next() {
					if line.as_ref().is_ok_and(|line| line.starts_with("0x")) {
						let address:u64 = u64::from_str_radix(&line.unwrap()[2..], 16).unwrap();
						addresses.push(address);
						if address == END_TAG {
							break;
						}
					}
				}
			}
			sleep(AWAIT_ADDRESS_PRINT_INTERVAL);
		}
		addresses
	}




	/* TEST METHODS */

	/// Test if memory_annihilator can read bytes and parse values from memory of external programs.
	fn read_process_memory(address:u64) {

		// Read memory.
		let mut pmm:ProcessMemoryManipulator64 = ProcessMemoryManipulator64::new(TEST_PROGRAM_EXE_NAME, false);
		let bytes:Vec<u8> = pmm.read_bytes(address, 4).expect("Could not read memory address");
		let value:u32 = pmm.read(address).expect("Could not read memory address");

		// Validate value.
		const EXPECTED_VALUE:u32 = 1618;
		assert_eq!(bytes, EXPECTED_VALUE.to_le_bytes());
		assert_eq!(value, EXPECTED_VALUE);
	}

	/// Test if memory_annihilator can write bytes and values to memory of external programs.
	fn write_process_memory(address_left:u64, address_right:u64, address_confirmation:u64) {

		// Read memory.
		let mut pmm:ProcessMemoryManipulator64 = ProcessMemoryManipulator64::new(TEST_PROGRAM_EXE_NAME, false);
		let left_value:u16 = pmm.read(address_left).expect("Could not get left value.");
		pmm.write::<u16>(address_right, left_value).expect("Could not write right value.");

		// Validate result.
		sleep(Duration::from_millis(2));
		assert_eq!(pmm.read::<u8, u64>(address_confirmation).expect("Could not read confirmation address."), 1);
	}

	/// Test if memory_annihilator can read bytes and parse values from memory of external programs.
	fn read_process_memory_from_pointer(address:u64, offset_1:u64, offset_2:u64) {

		// Read memory.
		let mut pmm:ProcessMemoryManipulator64 = ProcessMemoryManipulator64::new(TEST_PROGRAM_EXE_NAME, false);
		let bytes:Vec<u8> = pmm.read_bytes([address, offset_1, offset_2], 4).expect("Could not read memory address");
		let value:f32 = pmm.read([address, offset_1, offset_2]).expect("Could not read memory address");

		// Validate value.
		const EXPECTED_VALUE:f32 = 912.80;
		assert_eq!(bytes, EXPECTED_VALUE.to_le_bytes());
		assert_eq!(value, EXPECTED_VALUE);
	}
}