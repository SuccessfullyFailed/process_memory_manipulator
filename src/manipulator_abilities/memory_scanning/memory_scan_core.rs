use crate::{ AddressSourceType, MemoryDataType, MemorySnapshot, MemorySnapshotStorage, ProcessMemoryManipulator };
use std::{ error::Error, ops::Range, ptr, sync::{ Arc, Mutex, MutexGuard }, thread::{ self, JoinHandle } };



const RESULTS_LIST_GROWTH_SIZE:usize = 4096;



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



impl<AddressType:AddressSourceType + 'static> ProcessMemoryManipulator<AddressType> {

	/// Scan for a value with a value filter.
	pub fn scan<ValueType:MemoryDataType + 'static, ValueFilter:Fn(&ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		let snapshot:MemorySnapshot<AddressType> = self.create_memory_snapshot("", None)?;
		self.scan_with_snapshot(value_filter, snapshot)
	}

	/// Scan for a value with a value filter and the given snapshot.
	pub fn scan_with_snapshot<ValueType:MemoryDataType + 'static, ValueFilter:Fn(&ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter, snapshot:MemorySnapshot<AddressType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {

		// Skip scanning if address ranges are empty.
		if snapshot.address_ranges().is_empty() {
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

		// Spawn threads.
		let address_ranges:Arc<Vec<(Range<AddressType>, MemorySnapshotStorage)>> = Arc::new(snapshot.take_address_ranges());
		let range_cursor:Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
		let value_filter:Arc<ValueFilter> = Arc::new(value_filter);
		let mut threads:Vec<JoinHandle<Vec<(AddressType, ValueType)>>> = Vec::new();
		for _thread_index in 0..self.scanner_thread_count() {
			let thread_range_cursor:Arc<Mutex<usize>> = Arc::clone(&range_cursor);
			let thread_address_ranges:Arc<Vec<(Range<AddressType>, MemorySnapshotStorage)>> = Arc::clone(&address_ranges);
			let thread_value_filter:Arc<ValueFilter> = Arc::clone(&value_filter);
			threads.push(
				thread::spawn(move || {

					// Loop through memory ranges.
					let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
					let mut results_remaining_capacity:usize = RESULTS_LIST_GROWTH_SIZE;
					loop {

						// Increment cursor.
						let range_index:usize = {
							let mut cursor_handle:MutexGuard<'_, usize> = thread_range_cursor.lock().unwrap();
							*cursor_handle += 1;
							*cursor_handle - 1
						};
						if range_index >= thread_address_ranges.len() {
							break;
						}

						// Scan range.
						let (address_range, bytes_block) = &thread_address_ranges[range_index];
						if let Ok(bytes_block) = bytes_block.get_bytes() {
							let block_size:usize = bytes_block.len();
							if block_size > ValueType::BYTES_SIZE {

								// Loop through addresses in the snapshot data.
								let offset_end:usize = block_size - ValueType::BYTES_SIZE - 1;
								let value_pointer:*const ValueType::Bytes = &bytes_block[0] as *const u8 as *const ValueType::Bytes;
								for offset in 0..offset_end {

									// If a value matches the filter, add it to the results list.
									let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(value_pointer.byte_add(offset)) };
									let value:ValueType = bytes_to_value(value_bytes);
									if thread_value_filter(&value) {
										results.push((address_range.start + AddressType::from_usize(offset), value.clone()));
										results_remaining_capacity -= 1;
										if results_remaining_capacity == 0 {
											results.reserve(RESULTS_LIST_GROWTH_SIZE);
											results_remaining_capacity = RESULTS_LIST_GROWTH_SIZE;
										}
									}
								}
							}
						}
					}
					results
				})
			);
		}
		
		// Combine results from threads.
		let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
		for thread in threads {
			if let Ok(thread_results) = thread.join() {
				results.extend(thread_results);
			}
		}

		// Return results.
		Ok(MemoryScanResult::new(results))
	}



	/// Re-scan for a value with a filter on the current and previous value.
	pub fn re_scan<ValueType:MemoryDataType + 'static, ValueFilter:Fn(&ValueType, &ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		let snapshot:MemorySnapshot<AddressType> = self.create_memory_snapshot("", None)?;
		self.re_scan_with_snapshot(value_filter, previous_results, snapshot)
	}

	/// Re-scan for a value with a filter on the current and previous value and a snapshot.
	pub fn re_scan_with_snapshot<ValueType:MemoryDataType + 'static, ValueFilter:Fn(&ValueType, &ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter, previous_results:MemoryScanResult<AddressType, ValueType>, snapshot:MemorySnapshot<AddressType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		const BATCH_SIZE:usize = 4094;

		// Skip scanning if address ranges are empty.
		if snapshot.address_ranges().is_empty() {
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

		// Spawn threads.
		let address_ranges:Arc<Vec<(Range<AddressType>, MemorySnapshotStorage)>> = Arc::new(snapshot.take_address_ranges());
		let prev_results:Arc<Vec<(AddressType, ValueType)>> = Arc::new(previous_results.results);
		let batch_cursor:Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
		let value_filter:Arc<ValueFilter> = Arc::new(value_filter);
		let mut threads:Vec<JoinHandle<Vec<(AddressType, ValueType)>>> = Vec::new();
		for _thread_index in 0..self.scanner_thread_count() {
			let thread_batch_cursor:Arc<Mutex<usize>> = Arc::clone(&batch_cursor);
			let thread_address_ranges:Arc<Vec<(Range<AddressType>, MemorySnapshotStorage)>> = Arc::clone(&address_ranges);
			let thread_value_filter:Arc<ValueFilter> = Arc::clone(&value_filter);
			let thread_previous_results:Arc<Vec<(AddressType, ValueType)>> = Arc::clone(&prev_results);
			threads.push(
				thread::spawn(move || {

					// Loop through memory adress batches.
					let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
					let mut results_remaining_capacity:usize = RESULTS_LIST_GROWTH_SIZE;
					loop {

						// Increment cursor.
						let batch_index:usize = {
							let mut cursor_handle:MutexGuard<'_, usize> = thread_batch_cursor.lock().unwrap();
							*cursor_handle += 1;
							*cursor_handle - 1
						};
						let batch_start:usize = batch_index * BATCH_SIZE;
						if batch_start >= thread_previous_results.len() {
							break;
						}
						let batch_end:usize = (batch_start + BATCH_SIZE).min(thread_previous_results.len());
						
						// Scan batch.
						let batch:&[(AddressType, ValueType)] = &thread_previous_results[batch_start..batch_end];
						let mut cached_bytes_block:(Range<AddressType>, Vec<u8>, *const ValueType::Bytes) = (AddressType::default()..AddressType::default(), Vec::new(), ptr::null());
						for (address, previous_value) in batch {

							// If the address does not fall within the cached snapshot block, cache the required block.
							// As MemoryScanResults are inherently created in a way that largely sorts them, this should not happen often.
							if *address < cached_bytes_block.0.start || *address >= cached_bytes_block.0.end {
								if let Some(current_snapshot) = thread_address_ranges.iter().find(|(range, _)| range.start <= *address && range.end > *address) {
									if let Ok(bytes) = current_snapshot.1.get_bytes() {
										cached_bytes_block = (current_snapshot.0.clone(), bytes, ptr::null());
										cached_bytes_block.2 = &cached_bytes_block.1[0] as *const u8 as *const ValueType::Bytes;
									} else {
										continue;
									}
								} else {
									continue;
								}
							}

							// If the current and previous value matches the filter, add it to the results list.
							let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(cached_bytes_block.2.byte_add((*address - cached_bytes_block.0.start).to_usize())) };
							let value:ValueType = bytes_to_value(value_bytes);
							if thread_value_filter(&value, previous_value) {
								results.push((*address, value));
								results_remaining_capacity -= 1;
								if results_remaining_capacity == 0 {
									results.reserve(RESULTS_LIST_GROWTH_SIZE);
									results_remaining_capacity = RESULTS_LIST_GROWTH_SIZE;
								}
							}
						}
					}
					results
				})
			);
		}
		
		// Combine results from threads.
		let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
		for thread in threads {
			if let Ok(thread_results) = thread.join() {
				results.extend(thread_results);
			}
		}

		Ok(MemoryScanResult::new(results))
	}
}