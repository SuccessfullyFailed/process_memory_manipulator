#[cfg(test)]
mod tests {
	use crate::{ AOBInjection, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };



	#[test]
	fn test_aob_injection_small_replacement() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		// Create random instructions.
		let random_instructions:[u8; 7] = [
			0x01, 0x53, 0x12, // add [rbx+12], edx
			0x44, 0x8B, 0x4B, 0x12 // mov r9d, [rbx+12]
		];
		let replacement_instructions:[u8; 4] = [
			0x44, 0x8B, 0x4B, 0x12 // mov r9d, [rbx+12]
		];

		// Create and enable injection.
		let mut injection:AOBInjection<u64> = AOBInjection::new("01 53 12 44 8b 4b 12", replacement_instructions.to_vec()).unwrap();
		injection.enable(&mut pmm).unwrap();

		// Validate replacement.
		assert_eq!(random_instructions, [0x44, 0x8B, 0x4B, 0x12, 0x90, 0x90, 0x90]);
	}

	#[test]
	fn test_aob_injection_equal_replacement() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		// Create random instructions.
		let random_instructions:[u8; 7] = [
			0x01, 0x53, 0x13, // add [rbx+10], edx
			0x44, 0x8B, 0x4B, 0x13 // mov r9d, [rbx+13]
		];
		let replacement_instructions:[u8; 7] = [
			0x44, 0x8B, 0x4B, 0x13, // mov r9d, [rbx+13]
			0x01, 0x53, 0x13 // add [rbx+13], edx
		];

		// Create and enable injection.
		let mut injection:AOBInjection<u64> = AOBInjection::new("01 53 13 44 8b 4b 13", replacement_instructions.to_vec()).unwrap();
		injection.enable(&mut pmm).unwrap();

		// Validate replacement.
		assert_eq!(random_instructions, replacement_instructions);
	}

	#[test]
	fn test_aob_injection_empty_replacement() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		// Create random instructions.
		let random_instructions:[u8; 7] = [
			0x01, 0x53, 0x14, // add [rbx+14], edx
			0x44, 0x8B, 0x4B, 0x14 // mov r9d, [rbx+14]
		];

		// Create and enable injection.
		let mut injection:AOBInjection<u64> = AOBInjection::new("01 53 14 44 8b 4b 14", Vec::new()).unwrap();
		injection.enable(&mut pmm).unwrap();

		// Validate replacement.
		assert_eq!(random_instructions, [0x90; 7]);
	}

	/*
	#[test]
	fn test_aob_jmp_creation() {
		assert_eq!(AOBInjection::relative_direct_jmp(10_u32, 100_u32, false), vec![vec![0xE9], 85_i32.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::relative_direct_jmp(100_u32, 10_u32, false), vec![vec![0xE9], (-95_i32).to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::absolute_indirect_jmp(910_u32), vec![vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00], 910_u64.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		
		assert_eq!(AOBInjection::relative_direct_jmp(10_u64, 100_u64, false), vec![vec![0xE9], 85_i32.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::relative_direct_jmp(100_u64, 10_u64, false), vec![vec![0xE9], (-95_i32).to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::absolute_indirect_jmp(910_u64), vec![vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00], 910_u64.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		
		assert_eq!(AOBInjection::relative_direct_jmp(10_u32, 100_u32, true), vec![vec![0xE9], 85_i32.to_be_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::relative_direct_jmp(100_u32, 10_u32, true), vec![vec![0xE9], (-95_i32).to_be_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::absolute_indirect_jmp(910_u32), vec![vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00], 910_u64.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		
		assert_eq!(AOBInjection::relative_direct_jmp(10_u64, 100_u64, true), vec![vec![0xE9], 85_i32.to_be_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::relative_direct_jmp(100_u64, 10_u64, true), vec![vec![0xE9], (-95_i32).to_be_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
		assert_eq!(AOBInjection::absolute_indirect_jmp(910_u64), vec![vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00], 910_u64.to_le_bytes().to_vec()].into_iter().flatten().collect::<Vec<u8>>());
	}
	*/

	#[test]
	fn test_aob_injection_large_replacement() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		// Create random instructions.
		let random_instructions:[u8; 11] = [
			0x01, 0x53, 0x15, // add [rbx+15], edx
			0x44, 0x8B, 0x4B, 0x15, // mov r9d, [rbx+15]
			0x44, 0x8B, 0x4B, 0x15 // mov r9d, [rbx+15]
		];
		let replacement_instructions:[u8; 16] = [
			0x44, 0x8B, 0x4B, 0x15, // mov r9d, [rbx+15]
			0x44, 0x8B, 0x4B, 0x15, // mov r9d, [rbx+15]
			0x44, 0x8B, 0x4B, 0x15, // mov r9d, [rbx+15]
			0x44, 0x8B, 0x4B, 0x15 // mov r9d, [rbx+15]
		];
		let random_instructions_address:u64 = &random_instructions[0] as *const u8 as u64;

		// Create and enable injection.
		let mut injection:AOBInjection<u64> = AOBInjection::new("01 53 15 44 8B 4B 15 44 8B 4B 15", replacement_instructions.to_vec()).unwrap();
		injection.enable(&mut pmm).unwrap();

		// Validate that an external piece of code was created and the jump to it is accurate.
		assert_eq!(random_instructions[0], 0xE9);
		let reroute_offset:i32 = i32::from_le_bytes(random_instructions[1..5].try_into().unwrap());
		let reroute_start:u64 = (random_instructions_address as i64 + 5 + reroute_offset as i64) as u64;
		assert!(reroute_start != random_instructions_address, "Reroute address is same as injection address.");
		assert_eq!(pmm.read_bytes(reroute_start, replacement_instructions.len()).unwrap(), &replacement_instructions);

		// Validate the jump back is accurate.
		let reroute_trail_address:u64 = reroute_start + replacement_instructions.len() as u64;
		let reroute_trail:Vec<u8> = pmm.read_bytes(reroute_trail_address, 5).unwrap();
		assert_eq!(reroute_trail[0], 0xE9);
		let reroute_back_offset:i32 = i32::from_le_bytes(reroute_trail[1..].try_into().unwrap());
		let reroute_back_target_address:u64 = ((reroute_trail_address + 5) as i64 + reroute_back_offset as i64) as u64;
		assert_eq!(reroute_back_target_address, random_instructions_address + random_instructions.len() as u64);
	}
}