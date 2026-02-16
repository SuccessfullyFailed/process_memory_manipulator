use crate::{ AOBReference, AddressSourceType, ProcessMemoryManipulator, RawAobPattern };
use std::error::Error;



pub struct AOBInjection<AddressType:AddressSourceType> {
	search_pattern:RawAobPattern,
	replacement_bytes:Vec<u8>,
	original_bytes:Option<Vec<u8>>,
	injection_address:Option<AddressType>,
	reroute_injection_address:Option<AddressType>
}
impl<AddressType:AddressSourceType + 'static> AOBInjection<AddressType> {

	/* CONSTRUCTION METHODS */

	/// Create a new AOB injection.
	pub fn new<AOBRef:AOBReference>(pattern:AOBRef, replacement_bytes:Vec<u8>) -> Result<AOBInjection<AddressType>, Box<dyn Error>> {
		Ok(AOBInjection {
			search_pattern: pattern.into_aob()?,
			replacement_bytes,
			original_bytes: None,
			injection_address: None,
			reroute_injection_address: None
		})
	}


	/* USAGE METHODS */

	/// Inject the injection.
	pub fn enable(&mut self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<(), Box<dyn Error>> {
		const DO_NOTHING_BYTE:u8 = 0x90;

		// If already injected, ignore and return success.
		if self.injection_address.is_some() {
			return Ok(());
		}

		// Find AOB pattern in memory.
		match pmm.scan_aob(self.search_pattern.clone())? {
			Some(injection_address) => {
				let search_pattern_len:usize = self.search_pattern.len();
				let replace_pattern_len:usize = self.replacement_bytes.len();
				self.original_bytes = Some(pmm.read_bytes(injection_address, search_pattern_len)?);
				self.injection_address = Some(injection_address);

				// If no replacement, create empty instruction address.
				if replace_pattern_len == 0 {
					pmm.write_bytes(injection_address, &vec![DO_NOTHING_BYTE; search_pattern_len])?;
					Ok(())
				}

				// If replacement pattern fits inside existing pattern, simply replace in memory.
				else if search_pattern_len >= replace_pattern_len {
					pmm.write_bytes(injection_address, &self.replacement_bytes)?;
					let empty_space:usize = search_pattern_len - replace_pattern_len;
					if empty_space > 0 {
						pmm.write_bytes(injection_address + AddressSourceType::from_usize(replace_pattern_len), &vec![DO_NOTHING_BYTE; empty_space])?;
					}
					Ok(())
				}
				
				// If replacement pattern does not fit inside existing pattern, write bytes somewhere else and create a jump to and from that.
				else if search_pattern_len > 5 {
					let end_of_injection_address:AddressType = injection_address + AddressSourceType::from_usize(search_pattern_len);
					let big_endian:bool = pmm.big_endian();

					// Find or get reroute address.
					let rerouting_address:AddressType = match self.reroute_injection_address.clone() {
						Some(existing_rerouting_address) => existing_rerouting_address,
						None => {
							let required_memory:AddressType = AddressType::from_usize((((replace_pattern_len + 12) / 8) + 1) * 8); // Replacement bytes + biggest jump command (12) rounded by 8-bytes
							let mut reroute_bytes:Vec<u8> = self.replacement_bytes.clone();
							let reroute_address:AddressType = {
								if let Ok(reroute_near_address) = pmm.allocate_memory_near(required_memory, injection_address, AddressType::max_relative_jmp_offset()) {
									reroute_bytes.extend(Self::relative_direct_jmp(reroute_near_address + AddressType::from_usize(replace_pattern_len), end_of_injection_address, big_endian));
									reroute_near_address
								} else {
									let reroute_far_address:AddressType = pmm.allocate_memory(required_memory)?;
									reroute_bytes.extend(Self::absolute_indirect_jmp(end_of_injection_address));
									reroute_far_address
								}
							};
							pmm.write_bytes(reroute_address, &reroute_bytes)?;
							self.reroute_injection_address = Some(reroute_address.clone());
							reroute_address
						}
					};

					// Create reroute.
					let mut injection_bytes:Vec<u8> = Self::create_jmp(injection_address, rerouting_address, big_endian);
					if injection_bytes.len() > search_pattern_len {
						return Err("Injection does not fit within the search pattern, so replacement method is not possible.".into());
					}
					injection_bytes.extend(vec![DO_NOTHING_BYTE; search_pattern_len - injection_bytes.len()]);
					pmm.write_bytes(injection_address, &injection_bytes)?;
					Ok(())
				}

				// No suitable injection could be made.
				else {
					Err("Replacement pattern needs to be at least 5 bytes.".into())
				}
			},
			None => Err("Could not find memory reference using AOB pattern.".into())
		}
	}

	/// Disable the injection.
	pub fn disable(&mut self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<(), Box<dyn Error>> {
		match self.injection_address.clone() {
			Some(injection_address) => match self.original_bytes.take() {

				// Address and original bytes found, restore original bytes.
				Some(original_bytes) => {
					pmm.write_bytes(injection_address, &original_bytes)?;
					self.injection_address = None;
					Ok(())
				},

				// Address was found, but bytes weren't, return error.
				None => {
					Err("Could not find original bytes of found injection address.".into())
				}
			},

			// Was not injected yet, return success.
			None => Ok(())
		}
	}



	/* BYTE COMMANDS CREATION METHODS */

	/// Create a list of bytes that will jmp from one address to another. Automatically picks a relative-direct-jmp or an absolute-indirect-jmp The instruction address is the address where the jmp command starts, not the instruction after it.
	pub(crate) fn create_jmp(instruction_address:AddressType, target_address:AddressType, big_endian:bool) -> Vec<u8> {
		let jump_start:AddressType = instruction_address + AddressType::from_usize(5);
		let jump_offset_abs = {
			if jump_start > target_address {
				jump_start - target_address
			} else {
				target_address - jump_start
			}
		};
		if jump_offset_abs < AddressType::max_relative_jmp_offset() {
			Self::relative_direct_jmp(instruction_address, target_address, big_endian)
		} else {
			Self::absolute_indirect_jmp(target_address)
		}
	}

	/// Create a list of bytes that will do a relative jmp from one address to another. The instruction address is the address where the jmp command starts, not the instruction after it.
	pub(crate) fn relative_direct_jmp(instruction_address:AddressType, target_address:AddressType, big_endian:bool) -> Vec<u8> {
		const JMP_BYTE:u8 = 0xE9;
		const JMP_INSTRUCTION_BYTES_LEN:usize = 5;

		let jmp_origin:AddressType = instruction_address + AddressType::from_usize(JMP_INSTRUCTION_BYTES_LEN);
		let jmp_offset:AddressType = target_address.wrapping_sub(jmp_origin);
		let jmp_offset_bytes:Vec<u8> = {
			if big_endian {
				if AddressType::BYTES_SIZE == 4 {
					jmp_offset.mdt_to_be_bytes_vec()
				} else {
					jmp_offset.mdt_to_be_bytes_vec()[4..].to_vec()
				}
			} else {
				if AddressType::BYTES_SIZE == 4 {
					jmp_offset.mdt_to_le_bytes_vec()
				} else {
					jmp_offset.mdt_to_le_bytes_vec()[..4].to_vec()
				}
			}
		};
		vec![vec![JMP_BYTE], jmp_offset_bytes].into_iter().flatten().collect()
	}

	/// Create a list of bytes that will do an absolute jmp to the target address.
	pub(crate) fn absolute_indirect_jmp(target_address:AddressType) -> Vec<u8> {
		const JUMP_BYTE:u8 = 0xFF;
		const QWORD_BYTE:u8 = 0x25;

		let address_bytes:Vec<u8> = {
			let mut address_bytes:Vec<u8> = target_address.mdt_to_le_bytes_vec(); // x86 always uses little endian for absolute jumps.
			address_bytes.extend(vec![0x00; 8 - address_bytes.len()]);
			address_bytes
		};
		if address_bytes.len() == 4 {
			vec![vec![JUMP_BYTE, QWORD_BYTE], address_bytes].into_iter().flatten().collect()
		} else {
			vec![vec![JUMP_BYTE, QWORD_BYTE, 0x00, 0x00, 0x00, 0x00], address_bytes].into_iter().flatten().collect()
		}
	}
}