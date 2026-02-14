use crate::{ AddressSourceType, MemoryDataType, MemorySnapshot, ProcessMemoryManipulator };
use std::{ error::Error };



pub type RawAobPattern = Vec<Option<u8>>;
pub trait AOBReference {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>>;
}
impl AOBReference for Vec<Option<u8>> {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		Ok(self)
	}
}
impl AOBReference for Vec<u8> {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		Ok(self.into_iter().map(|value| Some(value)).collect())
	}
}
impl<const SIZE:usize> AOBReference for [Option<u8>; SIZE] {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		self.to_vec().into_aob()
	}
}
impl<const SIZE:usize> AOBReference for [u8; SIZE] {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		self.to_vec().into_aob()
	}
}
impl AOBReference for String {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		self.as_str().into_aob()
	}
}
impl AOBReference for &str {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		const CHAR_TO_VALUE:fn(char) -> Result<u8, Box<dyn Error>> = |character:char| {
			if character >= '0' && character <= '9' {
				Ok(character as u8 - '0' as u8)
			} else if character >= 'a' && character <= 'f' {
				Ok(10 + character as u8 - 'a' as u8)
			} else if character >= 'A' && character <= 'F' {
				Ok(10 + character as u8 - 'A' as u8)
			} else {
				return Err(format!("Could not create AOB from character '{character}' as it is not 1-byte a hex value.").into())
			}
		};

		let mut bytes:RawAobPattern = Vec::new();
		for word in self.split(" ").filter(|word| !word.is_empty()).map(|word| if word.starts_with("0x") { &word[2..] } else { word }) {
			let word_chars:Vec<char> = word.chars().collect();
			if word_chars.len() != 2 {
				return Err(format!("Could not create AOB from '{word}', AOB words are supposed to be 2 characters long.").into());
			}
			if word == "??" {
				bytes.push(None);
			} else {
				bytes.push(Some(CHAR_TO_VALUE(word_chars[0])? * 16 + CHAR_TO_VALUE(word_chars[1])?));
			}
		}
		
		if bytes.is_empty() {
			return Err(format!("Could not create AOB from {self}, returned empty list of bytes.").into());
		}

		Ok(bytes)
	}
}



pub struct AOBInjection<AddressType:AddressSourceType> {
	search_pattern:RawAobPattern,
	replacement_bytes:Vec<u8>,
	original_bytes:Option<Vec<u8>>,
	injection_address:Option<AddressType>,
	reroute_injection_address:Option<AddressType>
}
impl<AddressType:AddressSourceType> AOBInjection<AddressType> {

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

	/// Inject the injection.
	pub fn enable(&mut self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<(), Box<dyn Error>> where AddressType:MemoryDataType + PartialOrd {
		const JUMP_BYTE:u8 = 0xE9;
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
					let end_of_injection_address:AddressType  = injection_address + AddressSourceType::from_usize(replace_pattern_len);

					// Find or get reroute address.
					let rerouting_address:AddressType = match self.reroute_injection_address.clone() {
						Some(existing_rerouting_address) => existing_rerouting_address,
						None => {

							// Create memory for reroute.
							let required_memory:usize = (((replace_pattern_len + 5) / 8) + 1) * 8; // Replacement bytes + jump command (5) rounded by 8-bytes
							let reroute_address:AddressType = pmm.allocate_memory_near(AddressType::from_usize(required_memory), injection_address)?;
							self.reroute_injection_address = Some(reroute_address.clone());

							// Create external function.
							let end_of_reroute:AddressType = reroute_address + AddressType::from_usize(replace_pattern_len) + AddressType::from_usize(replace_pattern_len + 5);
							let jump_from_reroute_offset:AddressType = end_of_injection_address.wrapping_sub(end_of_reroute);
							let reroute_bytes:Vec<u8> = [self.replacement_bytes.clone(), vec![JUMP_BYTE], jump_from_reroute_offset.mdt_to_le_bytes_vec()].into_iter().flatten().collect();
							pmm.write_bytes(reroute_address, &reroute_bytes)?;

							// Return reroute address.
							reroute_address
						}
					};

					// Create reroute.
					let jump_to_reroute_offset:AddressType = rerouting_address.wrapping_sub(injection_address + AddressType::from_usize(5)); // Jump from instruction after this to the reroute.
					let to_reroute_jump_bytes:Vec<u8> = [vec![JUMP_BYTE], jump_to_reroute_offset.mdt_to_le_bytes_vec()].into_iter().flatten().collect();
					pmm.write_bytes(injection_address, &to_reroute_jump_bytes)?;
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
}



impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {

	/// Scan for an address using an AOB pattern.
	pub fn scan_aob<AOBRef:AOBReference>(&mut self, aob_reference:AOBRef) -> Result<Option<AddressType>, Box<dyn Error>> {
		let snapshot:MemorySnapshot<AddressType> = self.create_memory_snapshot("", None)?;
		self.scan_aob_with_snapshot(aob_reference, &snapshot)
	}

	/// Scan for an address using an AOB pattern and a snapshot.
	pub fn scan_aob_with_snapshot<AOBRef:AOBReference>(&mut self, aob_reference:AOBRef, snapshot:&MemorySnapshot<AddressType>) -> Result<Option<AddressType>, Box<dyn Error>> {
		let raw_aob:RawAobPattern = aob_reference.into_aob()?;
		let aob_len:usize = raw_aob.len();
		
		// Skip scanning if address ranges are empty.
		if snapshot.address_ranges().is_empty() {
			return Ok(None);
		}

		// Loop through addresses ranges of the snapshot.
		for (address_range, bytes_block) in snapshot.address_ranges() {
			let block_bytes:Vec<u8> = bytes_block.get_bytes()?;
			let block_size:usize = block_bytes.len();
			if block_size > aob_len {

				// Loop through addresses in the snapshot data.
				let offset_end:usize = block_size - aob_len - 1;
				for offset in 0..offset_end {

					// If a value matches the aob pattern, return the found address.
					let mut full_match:bool = true;
					for (left, right) in raw_aob.iter().zip(&block_bytes[offset..offset + aob_len]) {
						if let Some(left) = left {
							if left != right {
								full_match = false;
							}
						}
					}
					if full_match {
						return Ok(Some(address_range.start + AddressType::from_usize(offset)));
					}
				}
			}
		}

		// No value found.
		Ok(None)
	}
}