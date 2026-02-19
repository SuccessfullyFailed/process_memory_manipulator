use crate::{ AddressSourceType, MemoryDataType, ProcessMemoryManipulator, MemoryScanResult };
use std::{ error::Error, ops::{ Range, Sub } };



impl<AddressType:AddressSourceType + 'static> ProcessMemoryManipulator<AddressType> {

	/* SIMPLIFIED SCAN METHODS */

	/// Scan the memory for a specific value.
	pub fn scan_value_exact<ValueType:MemoryDataType + PartialEq + PartialOrd + 'static>(&mut self, value:ValueType) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.scan(move |found_value| found_value == &value)
	}

	/// Scan the memory for a value in a specific range.
	pub fn scan_value_between<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value_range:Range<ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.scan(move |found_value| found_value >= &value_range.start && found_value < &value_range.end)
	}

	/// Scan the memory for a value less than the given number.
	pub fn scan_value_less_than<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value:ValueType) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.scan(move |found_value| *found_value < value)
	}

	/// Scan the memory for a value greater than the given number.
	pub fn scan_value_greater_than<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value:ValueType) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.scan(move |found_value| *found_value > value)
	}



	/// Re-scan the memory for a specific value.
	pub fn re_scan_value_exact<ValueType:MemoryDataType + PartialEq + PartialOrd + 'static>(&mut self, value:ValueType, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, _previous_found_value| found_value == &value, previous_results)
	}

	/// Re-scan the memory for a value in a specific range.
	pub fn re_scan_value_between<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value_range:Range<ValueType>, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, _previous_found_value| found_value >= &value_range.start && found_value < &value_range.end, previous_results)
	}

	/// Re-scan the memory for a value less than the given number.
	pub fn re_scan_value_less_than<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value:ValueType, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, _previous_found_value| found_value < &value, previous_results)
	}

	/// Re-scan the memory for a value greater than the given number.
	pub fn re_scan_value_greater_than<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, value:ValueType, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, _previous_found_value| *found_value > value, previous_results)
	}

	/// Re-scan the memory for any values that are the same as they previously were.
	pub fn re_scan_value_unchanged<ValueType:MemoryDataType + PartialEq + PartialOrd + 'static>(&mut self, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, previous_found_value| found_value == previous_found_value, previous_results)
	}

	/// Re-scan the memory for any values that are not the same as they previously were.
	pub fn re_scan_value_changed<ValueType:MemoryDataType + PartialEq + PartialOrd + 'static>(&mut self, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, previous_found_value| found_value != previous_found_value, previous_results)
	}

	/// Re-scan the memory for any values that have increased since last scan.
	pub fn re_scan_value_increased<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, previous_found_value| found_value > previous_found_value, previous_results)
	}

	/// Re-scan the memory for any values that have increased by a certain amount since last scan.
	pub fn re_scan_value_increased_by<ValueType:MemoryDataType + PartialOrd + Sub<Output=ValueType> + 'static>(&mut self, increased_amount:ValueType, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, previous_found_value| found_value > &increased_amount && (*found_value - increased_amount) == *previous_found_value, previous_results)
	}

	/// Re-scan the memory for any values that have decreased since last scan.
	pub fn re_scan_value_decreased<ValueType:MemoryDataType + PartialOrd + 'static>(&mut self, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		self.re_scan(move |found_value, previous_found_value| found_value < previous_found_value, previous_results)
	}

	/// Re-scan the memory for any values that have increased by a certain amount since last scan.
	pub fn re_scan_value_decreased_by<ValueType:MemoryDataType + PartialOrd + Sub<Output=ValueType> + 'static>(&mut self, increased_amount:ValueType, previous_results:MemoryScanResult<AddressType, ValueType>) -> Result<MemoryScanResult<AddressType, ValueType>, Box<dyn Error>> {
		let pre_filtered_results:MemoryScanResult<AddressType, ValueType> = MemoryScanResult::new(previous_results.results.iter().filter(|(_address, value)| value > &increased_amount).cloned().collect());
		self.re_scan(move |found_value, previous_found_value| (*previous_found_value - increased_amount) == *found_value, pre_filtered_results)
	}
}