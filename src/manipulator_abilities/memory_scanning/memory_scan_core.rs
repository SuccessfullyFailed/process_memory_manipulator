use crate::{ AddressSourceType, MemoryAccessToken, MemoryDataType, MemoryIterator, ProcessMemoryManipulator, RegionIterator };
use std::{ ptr, error::Error, sync::{ Arc, Mutex, MutexGuard }, thread::{ self, JoinHandle } };
use urge_prique::WeighedPriorityQueue;



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
	pub fn scan<ValueType:MemoryDataType + PartialOrd + 'static, ValueFilter:Fn(&ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.scan_in(value_filter, RegionIterator::new())
	}

	/// Scan for a value with a value filter and the given snapshot.
	pub fn scan_in<ValueType:MemoryDataType + PartialOrd + 'static, ValueFilter:Fn(&ValueType) -> bool + Send + Sync + 'static, MemoryIter:MemoryIterator<AddressType> + 'static>(&mut self, value_filter:ValueFilter, memory_iterator:MemoryIter) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {

		// Create a short-hand bytes to value casting function.
		let bytes_to_value:fn(ValueType::Bytes) -> ValueType = {
			if self.big_endian() {
				ValueType::mdt_from_be_bytes
			} else {
				ValueType::mdt_from_le_bytes
			}
		};

		// Spawn threads.
		let memory_iterator:Arc<MemoryIter> = Arc::new(memory_iterator);
		let value_filter:Arc<ValueFilter> = Arc::new(value_filter);
		let mut threads:Vec<JoinHandle<Vec<(AddressType, ValueType)>>> = Vec::new();
		for thread_index in 0..self.scanner_thread_count() {
			let thread_memory_iterator:Arc<MemoryIter> = Arc::clone(&memory_iterator);
			let thread_value_filter:Arc<ValueFilter> = Arc::clone(&value_filter);
			let thread_process_name:String = self.process_name().to_string();
			let thread_big_endian:bool = self.big_endian();
			threads.push(
				thread::spawn(move || {

					// Create new memory manipulator thread.
					let mut thread_pmm:ProcessMemoryManipulator<AddressType> = ProcessMemoryManipulator::new(&thread_process_name, thread_big_endian);
					if let Err(error) = thread_pmm.open_handle(MemoryAccessToken::NONE) {
						eprintln!("Scanner thread {thread_index} could not open handle: {error}.");
					}

					// Loop through memory ranges.
					let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
					let mut results_remaining_capacity:usize = RESULTS_LIST_GROWTH_SIZE;
					loop {

						// Get next range.
						match thread_memory_iterator.next(&mut thread_pmm) {
							Err(error) => {
								eprintln!("[WARNING] Scanner thread {thread_index} failed getting next memory slice: {error}");
								continue
							},
							Ok(potential_memory_slice) => {
								match potential_memory_slice {
									None => break,
									Some(memory_slice) => {

										// Scan range.
										let slice_len:usize = memory_slice.bytes.len();
										if slice_len > ValueType::BYTES_SIZE {

											// Loop through addresses in the snapshot data.
											let offset_end:usize = slice_len - ValueType::BYTES_SIZE - 1;
											let value_pointer:*const ValueType::Bytes = &memory_slice.bytes[0] as *const u8 as *const ValueType::Bytes;
											for offset in 0..offset_end {

												// If a value matches the filter, add it to the results list.
												let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(value_pointer.byte_add(offset)) };
												let value:ValueType = bytes_to_value(value_bytes);
												if thread_value_filter(&value) {
													results.push((memory_slice.address_range.start + AddressType::from_usize(offset), value.clone()));
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
							}
						}
					}
					results
				})
			);
		}
		
		// Combine and return results.
		Ok(MemoryScanResult::new(
			Self::sort_result_lists(
				threads.into_iter().map(|join_handle| join_handle.join()).flatten().collect()
			)
		))
	}



	/// Re-scan for a value with a filter on the current and previous value.
	pub fn re_scan<ValueType:MemoryDataType + PartialOrd + 'static, ValueFilter:Fn(&ValueType, &ValueType) -> bool + Send + Sync + 'static>(&mut self, value_filter:ValueFilter, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan_in(value_filter, previous_results, RegionIterator::new())
	}

	/// Re-scan for a value with a filter on the current and previous value and a snapshot.
	pub fn re_scan_in<ValueType:MemoryDataType + PartialOrd + 'static, ValueFilter:Fn(&ValueType, &ValueType) -> bool + Send + Sync + 'static, MemoryIter:MemoryIterator<AddressType> + 'static>(&mut self, value_filter:ValueFilter, previous_results:MemoryScanResult<AddressType, ValueType>, memory_iterator:MemoryIter) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {

		// Create a short-hand bytes to value casting function.
		let bytes_to_value:fn(ValueType::Bytes) -> ValueType = {
			if self.big_endian() {
				ValueType::mdt_from_be_bytes
			} else {
				ValueType::mdt_from_le_bytes
			}
		};

		// Spawn threads.
		let memory_iterator = Arc::new(memory_iterator);
		let previous_results:Arc<Vec<(AddressType, ValueType)>> = Arc::new(previous_results.results);
		let previous_result_cursor:Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
		let value_filter:Arc<ValueFilter> = Arc::new(value_filter);
		let mut threads:Vec<JoinHandle<Vec<(AddressType, ValueType)>>> = Vec::new();
		for thread_index in 0..self.scanner_thread_count() {
			let thread_memory_iterator:Arc<MemoryIter> = Arc::clone(&memory_iterator);
			let thread_value_filter:Arc<ValueFilter> = Arc::clone(&value_filter);
			let thread_process_name:String = self.process_name().to_string();
			let thread_big_endian:bool = self.big_endian();
			let thread_previous_results:Arc<Vec<(AddressType, ValueType)>> = Arc::clone(&previous_results);
			let thread_previous_result_cursor:Arc<Mutex<usize>> = Arc::clone(&previous_result_cursor);
			threads.push(
				thread::spawn(move || {

					// Create new memory manipulator thread.
					let mut thread_pmm:ProcessMemoryManipulator<AddressType> = ProcessMemoryManipulator::new(&thread_process_name, thread_big_endian);
					if let Err(error) = thread_pmm.open_handle(MemoryAccessToken::NONE) {
						eprintln!("[WARNING] Re-scanner thread {thread_index} could not open handle: {error}.");
					}

					// Loop through memory address batches.
					let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(RESULTS_LIST_GROWTH_SIZE);
					let mut results_remaining_capacity:usize = RESULTS_LIST_GROWTH_SIZE;
					loop {

						// Get next range.
						let mut cursor_handle:MutexGuard<'_, usize> = thread_previous_result_cursor.lock().unwrap();
						let mut cursor:usize = *cursor_handle;
						if cursor >= thread_previous_results.len() {
							break;
						}
						let next_address:&AddressType = &thread_previous_results[cursor].0;
						let previous_results_start_index:usize = cursor;
						match thread_memory_iterator.next_after(*next_address, &mut thread_pmm) {

							// If next range errored, print error and move on to next result in previous results list.
							Err(error) => {
								cursor += 1;
								*cursor_handle = cursor;
								drop(cursor_handle);
								eprintln!("Re-scanner thread {thread_index} failed getting next memory slice: {error}");
							},
							Ok(potential_memory_slice) => match potential_memory_slice {

								// If no next range is available, stop looking.
								None => {
									break;
								},

								// If next range is available, skip cursor past each previous result in the memory slice and parse the slice.
								Some(memory_slice) => {

									// skip cursor past each previous result in the memory slice.
									let address_range_end:AddressType = memory_slice.address_range.end;
									while cursor < thread_previous_results.len() && thread_previous_results[cursor].0 < address_range_end {
										cursor += 1;
									}
									*cursor_handle = cursor;
									let previous_results_end_index:usize = cursor;
									drop(cursor_handle);

									// Scan addresses in range.
									let address_range_start:AddressType = memory_slice.address_range.start;
									let value_bytes_pointer:*const ValueType::Bytes = &memory_slice.bytes[0] as *const u8 as *const ValueType::Bytes;
									for previous_result_index in previous_results_start_index..previous_results_end_index {
										let (address, previous_value) = &thread_previous_results[previous_result_index];
										if *address < address_range_start {
											continue;
										}
										let offset:AddressType = *address - address_range_start;
										
										// If the current and previous value matches the filter, add it to the results list.
										let value_bytes:ValueType::Bytes = unsafe { ptr::read_unaligned(value_bytes_pointer.byte_add(offset.to_usize())) };
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
							}
						}
					}
					results
				})
			);
		}
		
		// Combine and return results.
		Ok(MemoryScanResult::new(
			Self::sort_result_lists(
				threads.into_iter().map(|join_handle| join_handle.join()).flatten().collect()
			)
		))
	}



	/// Sort a list of result lists into one list.
	pub fn sort_result_lists<ValueType:MemoryDataType + PartialOrd>(results_lists:Vec<Vec<(AddressType, ValueType)>>) -> Vec<(AddressType, ValueType)> {
		let total_result_count:usize = results_lists.iter().map(|list| list.len()).sum();
		let mut results:Vec<(AddressType, ValueType)> = Vec::with_capacity(total_result_count);
		let mut priority_queue = WeighedPriorityQueue::new(|(address, _result_list_index, _element_index)| *address);

		// Initial population
		for (result_list_index, result_list) in results_lists.iter().enumerate() {
			if let Some(&first_value) = result_list.first() {
				priority_queue.push((first_value, result_list_index, 0));
			}
		}

		// Keep taking the smallest value from the list and adding the next value from the list it was taken from.
		while let Some((value, result_list_index, element_index)) = priority_queue.pop() {
			results.push(value);
			let next_element_index:usize = element_index + 1;
			if let Some(&next_value) = results_lists[result_list_index].get(next_element_index) {
				priority_queue.push((next_value, result_list_index, next_element_index));
			}
		}

		// Return results.
		results
	}
}