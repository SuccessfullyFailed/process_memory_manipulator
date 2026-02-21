#[cfg(test)]
mod tests {
	use crate::{ AOBReference, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };
	use mini_rand::RandomNumber;



	#[test]
	fn test_option_bytes_vec_to_aob() {
		assert_eq!(
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)].into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_option_bytes_array_to_aob() {
		assert_eq!(
			[Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)].into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_bytes_vec_to_aob() {
		assert_eq!(
			vec![0x00, 0xFF, 0x08, 0x1F, 0x22, 0x01, 0x01].into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), Some(0x08), Some(0x1F), Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_bytes_array_to_aob() {
		assert_eq!(
			[0x00, 0xFF, 0x08, 0x1F, 0x22, 0x01, 0x01].into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), Some(0x08), Some(0x1F), Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_string_to_aob() {
		assert_eq!(
			"00 FF ?? 08 1F ?? 22 01 01".to_string().into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_str_to_aob() {
		assert_eq!(
			"00 FF ?? 08 1F ?? 22 01 01".into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_str_with_prefix_to_aob() {
		assert_eq!(
			"0x00 0xFF ?? 0x08 0x1F ?? 0x22 0x01 0x01".into_aob().unwrap().raw_bytes(),
			vec![Some(0x00), Some(0xFF), None, Some(0x08), Some(0x1F), None, Some(0x22), Some(0x01), Some(0x01)]
		);
	}

	#[test]
	fn test_invalid_str_to_aob() {
		assert!("0x00 0xFF ? 0x08 0x1F ?? 0x22 0x01 0x01".into_aob().is_err(), "AOB pattern with single question mark did not error.");
		assert!("0x00 0xFFE ?? 0x08 0x1F ?? 0x22 0x01 0x01".into_aob().is_err(), "AOB pattern with three-character word did not error.");
		assert!("0x00 0xF ?? 0x08 0x1F ?? 0x22 0x01 0x01".into_aob().is_err(), "AOB pattern with one-character word did not error.");
		assert!("0x0x00 0xFF ?? 0x08 0x1F ?? 0x22 0x01 0x01".into_aob().is_err(), "AOB pattern with double hex prefix did not error.");
		assert!("".into_aob().is_err(), "Empty AOB pattern did not error.");
	}



	#[test]
	fn test_aob_scan_for_pattern_exact() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		let mut random_data_block:[u8; 128] = [0; 128];
		for index in 0..128 {
			random_data_block[index] = u8::random();
		}
		let random_data_block_address:u64 = &random_data_block[0] as *const u8 as u64;
		let aob_pattern:Vec<u8> = random_data_block[..100].to_vec();

		assert_eq!(pmm.scan_aob(aob_pattern.clone()).unwrap(), Some((random_data_block_address, aob_pattern)));
	}

	#[test]
	fn test_aob_scan_for_pattern_with_gaps() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		let mut random_data_block:[u8; 128] = [0; 128];
		for index in 0..128 {
			random_data_block[index] = u8::random();
		}
		let random_data_block_address:u64 = &random_data_block[0] as *const u8 as u64;
		let aob_pattern:Vec<u8> = random_data_block[..100].to_vec();
		let aob_pattern:Vec<Option<u8>> = aob_pattern.iter().enumerate().map(|(index, value)| if [3, 6, 21, 42, 84].contains(&index) { None } else { Some(*value) }).collect();

		assert_eq!(pmm.scan_aob(aob_pattern).unwrap(), Some((random_data_block_address, random_data_block[..100].to_vec())));
	}

	#[test]
	fn test_aob_scan_for_pattern_str_with_gaps() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);
		
		let mut random_data_block:[u8; 128] = [0; 128];
		for index in 0..128 {
			random_data_block[index] = u8::random();
		}
		let random_data_block_address:u64 = &random_data_block[0] as *const u8 as u64;
		let aob_pattern:Vec<u8> = random_data_block[..100].to_vec();
		let aob_str:String = aob_pattern.iter().enumerate().map(|(index, value)| if [3, 6, 21, 42, 84].contains(&index) { "??".to_string() } else { format!("{:#04x}", value) }).collect::<Vec<String>>().join(" ");

		assert_eq!(pmm.scan_aob(aob_str).unwrap(), Some((random_data_block_address, random_data_block[..100].to_vec())));
	}
}