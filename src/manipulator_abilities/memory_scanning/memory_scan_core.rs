use crate::{ AddressSourceType, MemoryDataType, MemorySnapshot, ProcessMemoryManipulator };
use std::{ ptr, error::Error, ops::Range };



pub struct MemoryScanResult<AddressType:AddressSourceType, ValueType:MemoryDataType> {
	pub(crate) results:Vec<(AddressType, ValueType)>
}
impl<AddressType:AddressSourceType, ValueType:MemoryDataType> MemoryScanResult<AddressType, ValueType> {

	/// Create a new results.
	pub fn new(results:Vec<(AddressType, ValueType)>) -> MemoryScanResult<AddressType, ValueType> {
		MemoryScanResult {
			results
		}
	}

	/// Get the results of the scan.
	pub fn results(&self) -> &[(AddressType, ValueType)] {
		&self.results
	}
}



impl<AddressType:AddressSourceType + PartialEq + PartialOrd> ProcessMemoryManipulator<AddressType> {

	/// Scan for a value with a value filter.
	pub fn scan<ValueType:MemoryDataType, ValueFilter:Fn(&ValueType) -> bool>(&mut self, value_filter:ValueFilter) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		let snapshot:MemorySnapshot<AddressType> = self.create_memory_snapshot("", None)?;
		self.scan_with_snapshot(value_filter, &snapshot)
	}

	/// Scan for a value with a value filter and the given snapshot.
	pub fn scan_with_snapshot<ValueType:MemoryDataType, ValueFilter:Fn(&ValueType) -> bool>(&mut self, value_filter:ValueFilter, snapshot:&MemorySnapshot<AddressType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		const LIST_GROWTH_INCREMENT_SIZE:usize = 4096;

		// Validate arguments.
		let arguments_invalid:bool = snapshot.address_ranges().is_empty();
		if arguments_invalid {
			return Ok(MemoryScanResult::new(Vec::new()));
		}

		// Create a short-hand bytes to value casting function.
		let bytes_to_value:fn(ValueType::Bytes) -> ValueType = {
			if self.big_endian() {
				ValueType::mdt_from_be_bytes
			} else {
				ValueType::mdt_from_le_bytes
			}
		};

		// Loop through addresses ranges of the snapshot.
		let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(LIST_GROWTH_INCREMENT_SIZE);
		let mut results_remaining_capacity:usize = LIST_GROWTH_INCREMENT_SIZE;
		for (address_range, bytes_block) in snapshot.address_ranges() {
			let block_bytes:Vec<u8> = bytes_block.get_bytes()?;
			let block_size:usize = block_bytes.len();
			if block_size > ValueType::BYTES_SIZE {

				// Loop through addresses in the snapshot data.
				let offset_end:usize = block_size - ValueType::BYTES_SIZE - 1;
				let value_pointer:*const ValueType::Bytes = &block_bytes[0] as *const u8 as *const ValueType::Bytes;
				for offset in 0..offset_end {

					// If a value matches the filter, add it to the results list.
					let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(value_pointer.byte_add(offset)) };
					let value:ValueType = bytes_to_value(value_bytes);
					if value_filter(&value) {
						results.push((address_range.start + AddressType::from_usize(offset), value.clone()));
						results_remaining_capacity -= 1;
						if results_remaining_capacity == 0 {
							results.reserve(LIST_GROWTH_INCREMENT_SIZE);
							results_remaining_capacity = LIST_GROWTH_INCREMENT_SIZE;
						}
					}
				}
			}
			drop(block_bytes);
		}

		// Return results.
		Ok(MemoryScanResult::new(results))
	}

	/// Re-scan for a value with a filter on the current and previous value.
	pub fn re_scan<ValueType:MemoryDataType, ValueFilter:Fn(&ValueType, &ValueType) -> bool>(&mut self, value_filter:ValueFilter, previous_results:&MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		let snapshot:MemorySnapshot<AddressType> = self.create_memory_snapshot("", None)?;
		self.re_scan_with_snapshot(value_filter, previous_results, &snapshot)
	}

	/// Re-scan for a value with a filter on the current and previous value and a snapshot.
	pub fn re_scan_with_snapshot<ValueType:MemoryDataType, ValueFilter:Fn(&ValueType, &ValueType) -> bool>(&mut self, value_filter:ValueFilter, previous_results:&MemoryScanResult<AddressType, ValueType>, snapshot:&MemorySnapshot<AddressType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		const LIST_GROWTH_INCREMENT_SIZE:usize = 4096;

		// Validate arguments.
		let arguments_invalid:bool = snapshot.address_ranges().is_empty() || previous_results.results.is_empty();
		if arguments_invalid {
			return Ok(MemoryScanResult::new(Vec::new()));
		}

		// Create a short-hand bytes to value casting function.
		let bytes_to_value:fn(ValueType::Bytes) -> ValueType = {
			if self.big_endian() {
				ValueType::mdt_from_be_bytes
			} else {
				ValueType::mdt_from_le_bytes
			}
		};

		// Loop through previous results, caching the current snapshot block.
		let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(LIST_GROWTH_INCREMENT_SIZE);
		let mut results_remaining_capacity:usize = LIST_GROWTH_INCREMENT_SIZE;
		let mut cached_bytes_block:(Range<AddressType>, Vec<u8>, *const ValueType::Bytes) = (AddressType::default()..AddressType::default(), Vec::new(), ptr::null());
		for (address, previous_value) in &previous_results.results {

			// If the address does not fall within the cached snapshot block, cache the required block.
			// As MemoryScanResults are inheritly created in a way that sorts them, this should not happen often.
			if *address < cached_bytes_block.0.start || *address >= cached_bytes_block.0.end {
				if let Some(current_snapshot) = snapshot.address_ranges().iter().find(|(range, _)| range.start <= *address && range.end > *address) {
					cached_bytes_block = (current_snapshot.0.clone(), current_snapshot.1.get_bytes()?, ptr::null());
					cached_bytes_block.2 = &cached_bytes_block.1[0] as *const u8 as *const ValueType::Bytes;
				} else {
					continue;
				}
			}

			// If the current and previous value matches the filter, add it to the results list.
			let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(cached_bytes_block.2.byte_add((*address - cached_bytes_block.0.start).to_usize())) };
			let value:ValueType = bytes_to_value(value_bytes);
			if value_filter(&value, previous_value) {
				results.push((*address, value));
				results_remaining_capacity -= 1;
				if results_remaining_capacity == 0 {
					results.reserve(LIST_GROWTH_INCREMENT_SIZE);
					results_remaining_capacity = LIST_GROWTH_INCREMENT_SIZE;
				}
			}

		}
		Ok(MemoryScanResult::new(results))
	}
}