use crate::{ AOBReference, AddressSourceType, MachineCode, ProcessMemoryManipulator, RawAobPattern };
use std::{ error::Error, ops::Range };



pub struct AOBInjection<AddressType:AddressSourceType> {
	search_pattern:RawAobPattern,
	pattern_overwrite_length:Option<usize>,
	replacement:Box<dyn Fn(Vec<u8>) -> MachineCode<AddressType> + Send + Sync + 'static>,
	original_bytes:Option<Vec<u8>>,
	injection_address:Option<AddressType>,
	new_code_address:Option<AddressType>
}
impl<AddressType:AddressSourceType + 'static> AOBInjection<AddressType> {

	/* CONSTRUCTION METHODS */

	/// Create a new AOB injection.
	pub fn new<AOBRef:AOBReference, ReplacementFn:Fn(Vec<u8>) -> MachineCode<AddressType> + Send + Sync + 'static>(pattern:AOBRef, replacement:ReplacementFn) -> Result<AOBInjection<AddressType>, Box<dyn Error>> {
		Ok(AOBInjection {
			search_pattern: pattern.into_aob()?,
			pattern_overwrite_length: None,
			replacement: Box::new(replacement),
			original_bytes: None,
			injection_address: None,
			new_code_address: None
		})
	}

	/// Return self with a specific length of pattern to be overwritten.
	pub fn with_overwrite_length(mut self, length:usize) -> Self {
		self.pattern_overwrite_length = Some(length);
		self
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
			Some((injection_address, found_bytes)) => {
				let overwrite_length:usize = self.pattern_overwrite_length.unwrap_or(self.search_pattern.len());
				let replacement_bytes:MachineCode<AddressType> = (self.replacement)(found_bytes[..overwrite_length].to_vec());
				let replace_pattern_len:Range<usize> = replacement_bytes.estimated_byte_count();
				self.original_bytes = Some(pmm.read_bytes(injection_address, overwrite_length)?);
				self.injection_address = Some(injection_address);
				let big_endian:bool = pmm.big_endian();

				// If no replacement, create empty instruction address.
				if replace_pattern_len.end == 0 {
					pmm.write_bytes(injection_address, &vec![DO_NOTHING_BYTE; overwrite_length])?;
					Ok(())
				}

				// If replacement pattern fits inside existing pattern, simply replace in memory.
				else if replace_pattern_len.end <= overwrite_length {
					let replacement_bytes:Vec<u8> = replacement_bytes.clone().to_bytes(Some(injection_address), big_endian);
					let replace_pattern_len:usize = replacement_bytes.len();
					pmm.write_bytes(injection_address, &replacement_bytes)?;
					let empty_space:usize = overwrite_length - replace_pattern_len;
					if empty_space > 0 {
						pmm.write_bytes(injection_address + AddressSourceType::from_usize(replace_pattern_len), &vec![DO_NOTHING_BYTE; empty_space])?;
					}
					Ok(())
				}
				
				// If replacement pattern does not fit inside existing pattern, write bytes somewhere else and create a jump to and from that.
				else if overwrite_length > 5 {
					let big_endian:bool = pmm.big_endian();
					let end_of_injection_address:AddressType = injection_address + AddressSourceType::from_usize(overwrite_length);
					let reroute_function:MachineCode<AddressType> = replacement_bytes.clone() + MachineCode::jmp_to(end_of_injection_address);

					// Find or get reroute address.
					let rerouting_address:AddressType = match self.new_code_address.clone() {
						Some(existing_rerouting_address) => existing_rerouting_address,
						None => {
							let required_memory:AddressType = AddressType::from_usize(reroute_function.estimated_byte_count().end);
							let reroute_address:AddressType = pmm.allocate_memory_try_near(required_memory, injection_address, AddressType::max_relative_jmp_offset())?;
							pmm.write_bytes(reroute_address, &reroute_function.to_bytes(Some(reroute_address), big_endian))?;
							self.new_code_address = Some(reroute_address.clone());
							reroute_address
						}
					};

					// Create reroute.
					let injection_bytes:Vec<u8> = MachineCode::jmp_to(rerouting_address).to_bytes_amount(Some(injection_address), big_endian, overwrite_length);
					if injection_bytes.len() > overwrite_length {
						return Err("Injection does not fit within the search pattern, so replacement method is not possible.".into());
					}
					let empty_space:usize = overwrite_length - injection_bytes.len();
					pmm.write_bytes(injection_address, &[injection_bytes, vec![0x90; empty_space]].into_iter().flatten().collect::<Vec<u8>>())?;
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

	/// Disable the injection and remove the newly created memory.
	pub fn remove(&mut self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<(), Box<dyn Error>> {
		if let Some(created_code_address) = self.new_code_address {
			let original_bytes:Vec<u8> = self.original_bytes.clone().unwrap_or(Vec::new());
			let code_size:usize = (self.replacement)(original_bytes).estimated_byte_count().end;
			pmm.write_bytes(created_code_address, &vec![0x00; code_size])?;
		}
		self.disable(pmm)
	}



	/* PROPERTY GETTER METHODS */

	/// Whether or not the injection is enabled or not.
	pub fn is_enabled(&self) -> bool {
		self.injection_address.is_some()
	}

	/// If the injection is enabled, this will return the address found using the search pattern.
	pub fn injection_address(&self) -> Option<AddressType> {
		self.injection_address
	}

	/// If the injection is enabled, this will return the address where the new code is stored. Can be same as the injection address, but the injection could be a reroute to another address where the code is stored.
	pub fn new_code_address(&self) -> Option<AddressType> {
		self.new_code_address
	}
}