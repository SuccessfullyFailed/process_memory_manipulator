use crate::{ AddressSourceType, MemoryRegion, ProcessMemoryManipulator };
use std::{ error::Error, ops::Range, sync::{ Mutex, MutexGuard } };



pub struct MemorySlice<AddressType:AddressSourceType> {
	pub(crate) address_range:Range<AddressType>,
	pub(crate) bytes:Vec<u8>
}



pub trait MemoryIterator<AddressType:AddressSourceType>:Send + Sync {

	/// Get the next memory range to iterate over.
	fn next_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Option<Range<AddressType>>, Box<dyn Error>>;

	/// Get the next memory range with the accompanying bytes.
	fn next(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Option<MemorySlice<AddressType>>, Box<dyn Error>> {
		Ok(
			match self.next_range(pmm)? {
				Some(range) => Some(MemorySlice {
					address_range: range.clone(),
					bytes: pmm.read_bytes(range.start, (range.end - range.start).to_usize())?
				}),
				None => None
			}
		)
	}

	/// Skip to or past a specific address and return the range the target address falls in or in front of.
	fn next_after(&self, target_address:AddressType, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Option<MemorySlice<AddressType>>, Box<dyn Error>> {
		let mut result:Option<MemorySlice<AddressType>> = None;
		while result.is_none() {
			match self.next_range(pmm)? {
				Some(range) => {
					if range.end > target_address {
						result = Some(MemorySlice {
							address_range: range.clone(),
							bytes: pmm.read_bytes(range.start, (range.end - range.start).to_usize())?
						})
					}
				},
				None => break
			}
		}
		Ok(result)
	}
}



pub struct RegionIterator<AddressType:AddressSourceType> {
	region_validation_filter:Box<dyn Fn(&MemoryRegion<AddressType>) -> bool + Send + Sync + 'static>,
	cursor:Mutex<AddressType>,
	end:Option<AddressType>
}
impl<AddressType:AddressSourceType> RegionIterator<AddressType> {

	/// Create a new region iterator.
	pub fn new() -> RegionIterator<AddressType> {
		RegionIterator {
			region_validation_filter: Box::new(|region| region.is_readable()),
			cursor: Mutex::new(AddressType::from_usize(0)),
			end: None
		}
	}

	/// Return self with a specific range of addresses.
	pub fn with_range(mut self, range:Range<AddressType>) -> Self {
		*self.cursor.lock().unwrap() = range.start;
		self.end = Some(range.end);
		self
	}

	/// Return self with a specific region filter. Overrides the default filter, so make sure to include all validation.
	pub fn with_filter<RegionFilter:Fn(&MemoryRegion<AddressType>) -> bool + Send + Sync + 'static>(mut self, filter:RegionFilter) -> Self {
		self.region_validation_filter = Box::new(filter);
		self
	}
}
impl<AddressType:AddressSourceType> MemoryIterator<AddressType> for RegionIterator<AddressType> {
	
	/// Get the next memory range to iterate over.
	fn next_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Option<Range<AddressType>>, Box<dyn Error>> {
		let mut result:Option<Range<AddressType>> = None;
		let mut cursor_handle:MutexGuard<'_, AddressType> = self.cursor.lock().unwrap();
		let mut cursor:AddressType = *cursor_handle;
		while self.end.map(|end| cursor < end).unwrap_or(true) {
			if let Ok(region) = pmm.memory_region_at(cursor) {
				let region_start:AddressType = if region.base_address() < cursor { cursor } else { region.base_address() };
				let region_end:AddressType = region.base_address() + region.size();
				cursor = region_end;
				if (self.region_validation_filter)(&region) {
					result = Some(region_start..region_end);
					break;
				}
			} else {
				break;
			}
		}
		*cursor_handle = cursor;
		Ok(result)
	}
}