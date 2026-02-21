use crate::{ AOBReference, AddressSourceType, MachineCode, ProcessMemoryManipulator, RawAobPattern };
use std::{ error::Error, ops::Range };



pub struct AOBInjection<AddressType:AddressSourceType> {
	search_pattern:RawAobPattern,
	replacement:Box<dyn Fn(Vec<u8>) -> MachineCode<AddressType>>,
	original_bytes:Option<Vec<u8>>,
	injection_address:Option<AddressType>,
	reroute_injection_address:Option<AddressType>
}
impl<AddressType:AddressSourceType + 'static> AOBInjection<AddressType> {

	/* CONSTRUCTION METHODS */

	/// Create a new AOB injection.
	pub fn new<AOBRef:AOBReference, ReplacementFn:Fn(Vec<u8>) -> MachineCode<AddressType> + 'static>(pattern:AOBRef, replacement:ReplacementFn) -> Result<AOBInjection<AddressType>, Box<dyn Error>> {
		Ok(AOBInjection {
			search_pattern: pattern.into_aob()?,
			replacement: Box::new(replacement),
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
			Some((injection_address, found_bytes)) => {
				let search_pattern_len:usize = self.search_pattern.len();
				let replacement_bytes:MachineCode<AddressType> = (self.replacement)(found_bytes);
				let replace_pattern_len:Range<usize> = replacement_bytes.estimated_byte_count();
				self.original_bytes = Some(pmm.read_bytes(injection_address, search_pattern_len)?);
				self.injection_address = Some(injection_address);
				let big_endian:bool = pmm.big_endian();

				// If no replacement, create empty instruction address.
				if replace_pattern_len.end == 0 {
					pmm.write_bytes(injection_address, &vec![DO_NOTHING_BYTE; search_pattern_len])?;
					Ok(())
				}

				// If replacement pattern fits inside existing pattern, simply replace in memory.
				else if replace_pattern_len.end <= search_pattern_len {
					let replacement_bytes:Vec<u8> = replacement_bytes.clone().to_bytes(Some(injection_address), big_endian);
					let replace_pattern_len:usize = replacement_bytes.len();
					pmm.write_bytes(injection_address, &replacement_bytes)?;
					let empty_space:usize = search_pattern_len - replace_pattern_len;
					if empty_space > 0 {
						pmm.write_bytes(injection_address + AddressSourceType::from_usize(replace_pattern_len), &vec![DO_NOTHING_BYTE; empty_space])?;
					}
					Ok(())
				}
				
				// If replacement pattern does not fit inside existing pattern, write bytes somewhere else and create a jump to and from that.
				else if search_pattern_len > 5 {
					let big_endian:bool = pmm.big_endian();
					let end_of_injection_address:AddressType = injection_address + AddressSourceType::from_usize(search_pattern_len);
					let reroute_function:MachineCode<AddressType> = replacement_bytes.clone() + MachineCode::jmp_to(end_of_injection_address);

					// Find or get reroute address.
					let rerouting_address:AddressType = match self.reroute_injection_address.clone() {
						Some(existing_rerouting_address) => existing_rerouting_address,
						None => {
							let required_memory:AddressType = AddressType::from_usize(reroute_function.estimated_byte_count().end);
							let reroute_address:AddressType = pmm.allocate_memory_try_near(required_memory, injection_address, AddressType::max_relative_jmp_offset())?;
							pmm.write_bytes(reroute_address, &reroute_function.to_bytes(Some(reroute_address), big_endian))?;
							self.reroute_injection_address = Some(reroute_address.clone());
							reroute_address
						}
					};

					// Create reroute.
					let injection_bytes:Vec<u8> = MachineCode::jmp_to(rerouting_address).to_bytes_amount(Some(injection_address), big_endian, search_pattern_len);
					if injection_bytes.len() > search_pattern_len {
						return Err("Injection does not fit within the search pattern, so replacement method is not possible.".into());
					}
					let empty_space:usize = search_pattern_len - injection_bytes.len();
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
}