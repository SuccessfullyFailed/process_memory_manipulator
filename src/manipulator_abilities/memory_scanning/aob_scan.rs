use crate::{ AddressSourceType, MemoryAccessToken, MemoryIterator, ProcessMemoryManipulator, RegionIterator };
use std::{ sync::Arc, error::Error, thread::{ self, JoinHandle } };



#[derive(Clone)]
pub enum RawAobPattern { Full(Vec<u8>), Partial(Vec<Option<u8>>) }
impl RawAobPattern {

	/// Get the amount of bytes in the pattern.
	pub fn len(&self) -> usize {
		match self {
			RawAobPattern::Full(bytes) => bytes.len(),
			RawAobPattern::Partial(bytes) => bytes.len()
		}
	}

	/// Get the raw bytes of the pattern.
	pub fn raw_bytes(&self) -> Vec<Option<u8>> {
		match self {
			RawAobPattern::Full(bytes) => bytes.iter().map(|byte| Some(*byte)).collect(),
			RawAobPattern::Partial(bytes) => bytes.clone()
		}
	}
}



pub trait AOBReference {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>>;
}
impl AOBReference for RawAobPattern {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		Ok(self)
	}
}
impl AOBReference for Vec<Option<u8>> {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		Ok(RawAobPattern::Partial(self))
	}
}
impl AOBReference for Vec<u8> {
	fn into_aob(self) -> Result<RawAobPattern, Box<dyn Error>> {
		Ok(RawAobPattern::Full(self))
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

		let mut bytes:Vec<Option<u8>> = Vec::new();
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

		if bytes.iter().any(|byte| byte.is_none()) {
			Ok(RawAobPattern::Partial(bytes))
		} else {
			Ok(RawAobPattern::Full(bytes.into_iter().flatten().collect()))
		}
	}
}



impl<AddressType:AddressSourceType + 'static> ProcessMemoryManipulator<AddressType> {

	/// Scan for an address using an AOB pattern.
	pub fn scan_aob<AOBRef:AOBReference>(&mut self, aob_reference:AOBRef) -> Result<Option<(AddressType, Vec<u8>)>, Box<dyn Error>> {
		let raw_aob:RawAobPattern = aob_reference.into_aob()?;
		match raw_aob {
			RawAobPattern::Full(pattern) => self.scan_aob_with_snapshot_full_ref(pattern, RegionIterator::new()),
			RawAobPattern::Partial(pattern) => self.scan_aob_with_snapshot_partial_ref(pattern, RegionIterator::new())
		}
	}

	/// Scan for an address using an AOB pattern and a snapshot.
	pub fn scan_aob_with_snapshot_full_ref<MemoryIter:MemoryIterator<AddressType> + 'static>(&mut self, aob_pattern:Vec<u8>, memory_iterator:MemoryIter) -> Result<Option<(AddressType, Vec<u8>)>, Box<dyn Error>> {

		// Spawn threads.
		let memory_iterator:Arc<MemoryIter> = Arc::new(memory_iterator);
		let mut threads:Vec<JoinHandle<Option<(AddressType, Vec<u8>)>>> = Vec::new();
		for thread_index in 0..self.scanner_thread_count() {
			let thread_memory_iterator:Arc<MemoryIter> = Arc::clone(&memory_iterator);
			let thread_aob_pattern:Vec<u8> = aob_pattern.clone();
			let thread_process_name:String = self.process_name().to_string();
			let thread_big_endian:bool = self.big_endian();
			threads.push(
				thread::spawn(move || {
					let thread_aob_len:usize = thread_aob_pattern.len();

					// Create new memory manipulator thread.
					let mut thread_pmm:ProcessMemoryManipulator<AddressType> = ProcessMemoryManipulator::new(&thread_process_name, thread_big_endian);
					if let Err(error) = thread_pmm.open_handle(MemoryAccessToken::NONE) {
						eprintln!("AOB Scanner thread {thread_index} could not open handle: {error}.");
					}

					// Loop through memory ranges.
					loop {
						match thread_memory_iterator.next(&mut thread_pmm) {
							Err(error) => {
								eprintln!("[WARNING] AOB Scanner thread {thread_index} failed getting next memory slice: {error}");
								continue
							},
							Ok(potential_memory_slice) => {
								match potential_memory_slice {
									None => break,
									Some(memory_slice) => {

										// Loop through addresses in the snapshot data.
										let slice_len:usize = memory_slice.bytes.len();
										if slice_len > thread_aob_len {
											let offset_end:usize = slice_len - thread_aob_len - 1;
											for offset in 0..offset_end {

												// If a value matches the filter, return it as result and skip the cursor to the end.
												if memory_slice.bytes[offset..offset + thread_aob_len] == thread_aob_pattern {
													let absolute_address:AddressType = memory_slice.address_range.start + AddressType::from_usize(offset);
													let bytes:Vec<u8> = memory_slice.bytes[offset..offset + thread_aob_len].to_vec();
													return Some((absolute_address, bytes));
												}
											}
										}
									}
								}
							}
						}
					}

					None
				})
			);
		}
		
		// Find results from threads.
		let mut result:Option<(AddressType, Vec<u8>)> = None;
		for thread in threads {
			if let Ok(thread_result) = thread.join() {
				if let Some(thread_result) = thread_result {
					result = Some(thread_result);
					break;
				}
			}
		}
		Ok(result)
	}

	/// Scan for an address using an AOB pattern and a snapshot.
	pub fn scan_aob_with_snapshot_partial_ref<MemoryIter:MemoryIterator<AddressType> + 'static>(&mut self, aob_pattern:Vec<Option<u8>>, memory_iterator:MemoryIter) -> Result<Option<(AddressType, Vec<u8>)>, Box<dyn Error>> {

		// Find part of the aob pattern that can be fully matched.
		let fully_matchable:Vec<u8> = aob_pattern.iter().take_while(|byte| byte.is_some()).map(|byte| byte.unwrap()).collect();

		// Spawn threads.
		let memory_iterator:Arc<MemoryIter> = Arc::new(memory_iterator);
		let mut threads:Vec<JoinHandle<Option<(AddressType, Vec<u8>)>>> = Vec::new();
		for thread_index in 0..self.scanner_thread_count() {
			let thread_memory_iterator:Arc<MemoryIter> = Arc::clone(&memory_iterator);
			let thread_aob_fully_matchable:Vec<u8> = fully_matchable.clone();
			let thread_aob_partially_matchable:Vec<Option<u8>> = aob_pattern[thread_aob_fully_matchable.len()..].to_vec();
			let thread_process_name:String = self.process_name().to_string();
			let thread_big_endian:bool = self.big_endian();
			threads.push(
				thread::spawn(move || {
					let thread_aob_fully_matchable_len:usize = thread_aob_fully_matchable.len();
					let thread_aob_partially_matchable_len:usize = thread_aob_partially_matchable.len();
					let thread_aob_len:usize = thread_aob_fully_matchable_len + thread_aob_partially_matchable_len;

					// Create new memory manipulator thread.
					let mut thread_pmm:ProcessMemoryManipulator<AddressType> = ProcessMemoryManipulator::new(&thread_process_name, thread_big_endian);
					if let Err(error) = thread_pmm.open_handle(MemoryAccessToken::NONE) {
						eprintln!("AOB Scanner thread {thread_index} could not open handle: {error}.");
					}

					// Loop through memory ranges.
					loop {
						match thread_memory_iterator.next(&mut thread_pmm) {
							Err(error) => {
								eprintln!("[WARNING] AOB Scanner thread {thread_index} failed getting next memory slice: {error}");
								continue
							},
							Ok(potential_memory_slice) => {
								match potential_memory_slice {
									None => break,
									Some(memory_slice) => {

										// Loop through addresses in the snapshot data.
										let slice_len:usize = memory_slice.bytes.len();
										if slice_len > thread_aob_len {
											let offset_end:usize = slice_len - thread_aob_len - 1;
											for offset in 0..offset_end {

												// If a value matches the filter, return it as result and skip the cursor to the end.
												if memory_slice.bytes[offset..offset + thread_aob_fully_matchable_len] == thread_aob_fully_matchable {
													let mut full_match:bool = true;
													for (left, right) in thread_aob_partially_matchable.iter().zip(&memory_slice.bytes[offset + thread_aob_fully_matchable_len..offset + thread_aob_len]) {
														if let Some(left) = left {
															if left != right {
																full_match = false;
															}
														}
													}
													if full_match {
														let absolute_address:AddressType = memory_slice.address_range.start + AddressType::from_usize(offset);
														let bytes:Vec<u8> = memory_slice.bytes[offset..offset + thread_aob_len].to_vec();
														return Some((absolute_address, bytes));
													}
												}
											}
										}
									}
								}
							}
						};
					}
					None
				})
			);
		}
		
		// Find results from threads.
		let mut result:Option<(AddressType, Vec<u8>)> = None;
		for thread in threads {
			if let Ok(thread_result) = thread.join() {
				if let Some(thread_result) = thread_result {
					result = Some(thread_result);
					break;
				}
			}
		}
		Ok(result)
	}
}