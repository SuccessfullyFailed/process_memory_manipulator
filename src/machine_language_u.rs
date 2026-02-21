#[cfg(test)]
mod tests {
	use mini_rand::RandomNumber;
	use crate::{MachineCode, MemoryDataType};



	/* TEST INSTRUCTION COMBINING */

	#[test]
	fn test_add() {
		assert_eq!(
			(MachineCode::<u64>::raw_bytes(vec![0x10]) + MachineCode::raw_bytes(vec![0x20, 0x30])).to_bytes(None, false),
			vec![0x10, 0x20, 0x30]
		);
	}

	#[test]
	fn test_add_add() {
		assert_eq!(
			(MachineCode::<u64>::raw_bytes(vec![0x10]) + MachineCode::raw_bytes(vec![0x20, 0x30]) + MachineCode::raw_bytes(vec![0x40, 0x50])).to_bytes(None, false),
			vec![0x10, 0x20, 0x30, 0x40, 0x50]
		);
	}

	#[test]
	fn test_add_address_valid() {
		assert_eq!(
			(MachineCode::<u64>::jmp_to(0xA0) + MachineCode::<u64>::jmp_to(0xB0) + MachineCode::<u64>::jmp_to(0xC0)).to_bytes(Some(0), false),
			vec![
				0xE9, 0xA0 -  5, 0x00, 0x00, 0x00,
				0xE9, 0xB0 - 10, 0x00, 0x00, 0x00,
				0xE9, 0xC0 - 15, 0x00, 0x00, 0x00
			]
		);
	}

	#[test]
	fn test_mul() {
		assert_eq!(
			(MachineCode::<u64>::raw_bytes(vec![0x10]) * 5).to_bytes(None, false),
			vec![0x10; 5]
		);
	}

	#[test]
	fn test_mul_address_valid() {
		assert_eq!(
			(MachineCode::<u64>::jmp_to(0xA0) * 5).to_bytes(Some(0), false),
			vec![
				0xE9, 0xA0 -  5, 0x00, 0x00, 0x00,
				0xE9, 0xA0 - 10, 0x00, 0x00, 0x00,
				0xE9, 0xA0 - 15, 0x00, 0x00, 0x00,
				0xE9, 0xA0 - 20, 0x00, 0x00, 0x00,
				0xE9, 0xA0 - 25, 0x00, 0x00, 0x00,
			]
		);
	}



	/* TEST INSTRUCTION LITERALS */

	#[test]
	fn test_raw_bytes_u32() {
		let random_bytes:Vec<u8> = (0..17).map(|_| u8::random()).collect();
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u32> = MachineCode::<u32>::raw_bytes(random_bytes.clone());
				assert_eq!(
					machine_code.estimated_byte_count(),
					random_bytes.len()..random_bytes.len()
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					random_bytes
				);
			}
		}
	}
	#[test]
	fn test_raw_bytes_u64() {
		let random_bytes:Vec<u8> = (0..17).map(|_| u8::random()).collect();
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u64> = MachineCode::<u64>::raw_bytes(random_bytes.clone());
				assert_eq!(
					machine_code.estimated_byte_count(),
					random_bytes.len()..random_bytes.len()
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					random_bytes
				);
			}
		}
	}

	
	#[test]
	fn test_do_nothing_u32() {
		let bytes_length:usize = u8::random() as usize;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u32> = MachineCode::<u32>::do_nothing(bytes_length);
				assert_eq!(
					machine_code.estimated_byte_count(),
					bytes_length..bytes_length
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					vec![0x90; bytes_length]
				);
			}
		}
	}
	#[test]
	fn test_do_nothing_u64() {
		let bytes_length:usize = u8::random() as usize;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u64> = MachineCode::<u64>::do_nothing(bytes_length);
				assert_eq!(
					machine_code.estimated_byte_count(),
					bytes_length..bytes_length
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					vec![0x90; bytes_length]
				);
			}
		}
	}


	#[test]
	fn test_variable_u32() {
		let u16_variable:u16 = u16::random();
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let usize_variable_as_bytes:[u8; 2] = if big_endian { u16_variable.mdt_to_be_bytes() } else { u16_variable.mdt_to_le_bytes() };
				let machine_code:MachineCode<u32> = MachineCode::<u32>::variable(u16_variable);
				assert_eq!(
					machine_code.estimated_byte_count(),
					2..2
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					usize_variable_as_bytes
				);
			}
		}
	}
	#[test]
	fn test_variable_u64() {
		let u16_variable:u16 = u16::random();
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let usize_variable_as_bytes:[u8; 2] = if big_endian { u16_variable.mdt_to_be_bytes() } else { u16_variable.mdt_to_le_bytes() };
				let machine_code:MachineCode<u64> = MachineCode::<u64>::variable(u16_variable);
				assert_eq!(
					machine_code.estimated_byte_count(),
					2..2
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					usize_variable_as_bytes
				);
			}
		}
	}
	#[test]
	fn test_multiple_variables() {
		let u16_variable:u16 = u16::random();
		let float_variable:f32 = f32::random();
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let usize_variable_as_bytes:[u8; 2] = if big_endian { u16_variable.mdt_to_be_bytes() } else { u16_variable.mdt_to_le_bytes() };
				let float_variable_as_bytes:[u8; 4] = if big_endian { float_variable.mdt_to_be_bytes() } else { float_variable.mdt_to_le_bytes() };
				let machine_code:MachineCode<u32> = MachineCode::<u32>::variable(u16_variable) + MachineCode::<u32>::variable(float_variable);
				assert_eq!(
					machine_code.estimated_byte_count(),
					6..6
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					vec![
						usize_variable_as_bytes[0], usize_variable_as_bytes[1],
						float_variable_as_bytes[0], float_variable_as_bytes[1], float_variable_as_bytes[2], float_variable_as_bytes[3]
					]
				);
			}
		}
	}

	
	#[test]
	fn test_jmp_offset_u32() {
		let offset:i32 = i32::random() as i32;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				let offset_as_bytes:[u8; 4] = if big_endian { offset.to_be_bytes() } else { offset.to_le_bytes() };
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_offset(offset);
				assert_eq!(
					machine_code.estimated_byte_count(),
					5..5
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					vec![0xE9, offset_as_bytes[0], offset_as_bytes[1], offset_as_bytes[2], offset_as_bytes[3]]
				);
			}
		}
	}
	#[test]
	fn test_jmp_offset_u64() {
		let offset:i32 = i32::random() as i32;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				let offset_as_bytes:[u8; 4] = if big_endian { offset.to_be_bytes() } else { offset.to_le_bytes() };
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_offset(offset);
				assert_eq!(
					machine_code.estimated_byte_count(),
					5..5
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					vec![0xE9, offset_as_bytes[0], offset_as_bytes[1], offset_as_bytes[2], offset_as_bytes[3]]
				);
			}
		}
	}

	
	#[test]
	fn test_jmp_over_u32() {
		let random_contents:Vec<u8> = (0..u8::random()).map(|_| u8::random()).collect();
		let jmp_offset:i32 = random_contents.len() as i32;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				let offset_as_bytes:[u8; 4] = if big_endian { jmp_offset.to_be_bytes() } else { jmp_offset.to_le_bytes() };
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_over(MachineCode::RawBytes(random_contents.clone()));
				assert_eq!(
					machine_code.estimated_byte_count(),
					random_contents.len() + 5..random_contents.len() + 5
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					[
						vec![0xE9, offset_as_bytes[0], offset_as_bytes[1], offset_as_bytes[2], offset_as_bytes[3]],
						random_contents.clone()
					].into_iter().flatten().collect::<Vec<u8>>()
				);
			}
		}
	}
	#[test]
	fn test_jmp_over_u64() {
		let random_contents:Vec<u8> = (0..u8::random()).map(|_| u8::random()).collect();
		let jmp_offset:i32 = random_contents.len() as i32;
		for origin_address in [None, Some(10), Some(0xFF8844)] {
			for big_endian in [false, true] {
				let offset_as_bytes:[u8; 4] = if big_endian { jmp_offset.to_be_bytes() } else { jmp_offset.to_le_bytes() };
				println!("origin_address: {}\tbig_endian: {}", origin_address.map(|value| format!("{:#x}", value)).unwrap_or("None".to_string()), big_endian);
				let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_over(MachineCode::RawBytes(random_contents.clone()));
				assert_eq!(
					machine_code.estimated_byte_count(),
					random_contents.len() + 5..random_contents.len() + 5
				);
				assert_eq!(
					machine_code.to_bytes(origin_address, big_endian),
					[
						vec![0xE9, offset_as_bytes[0], offset_as_bytes[1], offset_as_bytes[2], offset_as_bytes[3]],
						random_contents.clone()
					].into_iter().flatten().collect::<Vec<u8>>()
				);
			}
		}
	}

	
	#[test]
	fn test_jmp_to_u32() {
		let target_address:u32 = 0xFF00;
		let source_address:u32 = 0xF000;
		let offset:i32 = target_address as i32 - source_address as i32 - 5;

		let full_offset_bytes_le:[u8; 4] = (target_address as i32).to_le_bytes();
		let relative_offset_bytes_le:[u8; 4] = offset.to_le_bytes();

		let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..6 // 5-byte offset jmp or 6-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(None, false),
			vec![0xFF, 0x25, full_offset_bytes_le[0], full_offset_bytes_le[1], full_offset_bytes_le[2], full_offset_bytes_le[3]]
		);


		let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..6 // 5-byte offset jmp or 6-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(None, true),
			vec![0xFF, 0x25, full_offset_bytes_le[3], full_offset_bytes_le[2], full_offset_bytes_le[1], full_offset_bytes_le[0]]
		);


		let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..6 // 5-byte offset jmp or 6-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address), false),
			vec![0xE9, relative_offset_bytes_le[0], relative_offset_bytes_le[1], relative_offset_bytes_le[2], relative_offset_bytes_le[3]]
		);


		let machine_code:MachineCode<u32> = MachineCode::<u32>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..6 // 5-byte offset jmp or 6-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address), true),
			vec![0xE9, relative_offset_bytes_le[3], relative_offset_bytes_le[2], relative_offset_bytes_le[1], relative_offset_bytes_le[0]]
		);
	}
	#[test]
	fn test_jmp_to_u64() {
		let target_address:u64 = u32::MAX as u64 + 0xFF00;
		let source_address_near:u64 = u32::MAX as u64 + 0xF000;
		let source_address_far:u64 = 0x00F0;
		let near_offset:i32 = target_address as i32 - source_address_near as i32 - 5;

		let target_address_bytes_le:[u8; 8] = target_address.to_le_bytes();
		let relative_offset_near_bytes_le:[u8; 4] = near_offset.to_le_bytes();

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(None, false),
			vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, target_address_bytes_le[0], target_address_bytes_le[1], target_address_bytes_le[2], target_address_bytes_le[3], target_address_bytes_le[4], target_address_bytes_le[5], target_address_bytes_le[6], target_address_bytes_le[7]]
		);

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address); // In x86, direct jump values are always 8-byte LE.
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(None, true),
			vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, target_address_bytes_le[0], target_address_bytes_le[1], target_address_bytes_le[2], target_address_bytes_le[3], target_address_bytes_le[4], target_address_bytes_le[5], target_address_bytes_le[6], target_address_bytes_le[7]]
		);

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address_near), false),
			vec![0xE9, relative_offset_near_bytes_le[0], relative_offset_near_bytes_le[1], relative_offset_near_bytes_le[2], relative_offset_near_bytes_le[3]]
		);

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address_near), true),
			vec![0xE9, relative_offset_near_bytes_le[3], relative_offset_near_bytes_le[2], relative_offset_near_bytes_le[1], relative_offset_near_bytes_le[0]]
		);

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address);
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address_far), false),
			vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, target_address_bytes_le[0], target_address_bytes_le[1], target_address_bytes_le[2], target_address_bytes_le[3], target_address_bytes_le[4], target_address_bytes_le[5], target_address_bytes_le[6], target_address_bytes_le[7]]
		);

		let machine_code:MachineCode<u64> = MachineCode::<u64>::jmp_to(target_address); // In x86, direct jump values are always 8-byte LE.
		assert_eq!(
			machine_code.estimated_byte_count(),
			5..14 // 5-byte offset jmp or 14-byte absolute jmp.
		);
		assert_eq!(
			machine_code.to_bytes(Some(source_address_far), true),
			vec![0xFF, 0x25, 0x00, 0x00, 0x00, 0x00, target_address_bytes_le[0], target_address_bytes_le[1], target_address_bytes_le[2], target_address_bytes_le[3], target_address_bytes_le[4], target_address_bytes_le[5], target_address_bytes_le[6], target_address_bytes_le[7]]
		);
	}
}