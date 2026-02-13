use crate::{ AddressSourceType, MemorySnapshot, ProcessMemoryManipulator };
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



impl<AddressType:AddressSourceType + PartialEq + PartialOrd> ProcessMemoryManipulator<AddressType> {

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