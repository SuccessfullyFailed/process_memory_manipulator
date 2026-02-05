use crate::{ AddressSourceType, MemoryAccessToken, MemoryRegion, ProcessMemoryManipulator };
use std::{ error::Error, ops::Range };
use mini_rand::RandomNumber;
use file_ref::FileRef;



const CACHE_FILE_EXTENSION:&str = "pmmssc";



pub struct MemorySnapshot<AddressType:AddressSourceType> {
	name:String,
	regions:Vec<(Range<AddressType>, MemorySnapshotStorage)>
}
impl<AddressType:AddressSourceType> MemorySnapshot<AddressType> {

	/* PROPERTY GETTER METHODS */

	/// Get the name of the snapshot.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the address ranges of the snapshot.
	pub fn address_ranges(&self) -> &[(Range<AddressType>, MemorySnapshotStorage)] {
		&self.regions
	}
}



pub enum MemorySnapshotStorage {
	Memory(Vec<u8>),
	File(FileRef)
}
impl MemorySnapshotStorage {

	/// Get the bytes of this source.
	pub fn get_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		match self {
			MemorySnapshotStorage::Memory(bytes) => Ok(bytes.clone()),
			MemorySnapshotStorage::File(file) => file.read_bytes()
		}
	}
}
impl Drop for MemorySnapshotStorage {
	fn drop(&mut self) {
		match self {
			MemorySnapshotStorage::Memory(_) => {},
			MemorySnapshotStorage::File(file) => {
				if file.exists() {
					let _ = file.delete();
				}
				if let Ok(cache_dir) = file.parent_dir() {
					if cache_dir.exists() {
						if cache_dir.list_files_recurse().is_empty() {
							let _ = cache_dir.delete();
						}
					}
				}
			}
		}
	}
}



impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {
	
	/// Create a memory snapshot of the process' entire memory to a specified directory.
	pub fn create_memory_snapshot(&mut self, snapshot_name:&str, cache_dir_path:Option<&str>) -> Result<MemorySnapshot<AddressType>, Box<dyn Error>> {
		const SNAPSHOT_CREATION_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::PROCESS_QUERY_INFORMATION.0 | MemoryAccessToken::READ_CONTROL.0 | MemoryAccessToken::PROCESS_VM_READ.0);

		self.open_handle(SNAPSHOT_CREATION_ACCESS)?;
		let memory_regions:Vec<MemoryRegion<AddressType>> = self.memory_regions()?;
		self.create_memory_snapshot_of(snapshot_name, cache_dir_path, memory_regions)
	}

	/// Create a memory snapshot of a specific range of addresses.
	pub fn create_memory_snapshot_of<Source:MemorySnapshotSource<AddressType>>(&mut self, snapshot_name:&str, cache_dir_path:Option<&str>, source:Source) -> Result<MemorySnapshot<AddressType>, Box<dyn Error>> {
		const SOURCED_SNAPSHOT_CREATION_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::READ_CONTROL.0 | MemoryAccessToken::PROCESS_VM_READ.0);

		// Prepare cache and handle.
		let cache_dir:Option<FileRef> = cache_dir_path.map(|path| FileRef::new(path));
		self.open_handle(SOURCED_SNAPSHOT_CREATION_ACCESS)?;

		// Read and store memory ranges.
		let address_ranges:Vec<Range<AddressType>> = source.as_raw_address_range(self)?;
		let mut snapshot_regions:Vec<(Range<AddressType>, MemorySnapshotStorage)> = Vec::with_capacity(address_ranges.len());
		for range in address_ranges {
			if let Ok(region_bytes) = self.read_bytes(range.start, (range.end - range.start).to_usize()) {
				snapshot_regions.push((
					range.clone(),
					match &cache_dir {
						Some(cache_dir) => {
							let mut region_file:FileRef = cache_dir.clone() + "/000." + CACHE_FILE_EXTENSION;
							while region_file.exists() {
								region_file = cache_dir.clone() + "/" + &u64::random().to_string() + "." + CACHE_FILE_EXTENSION;
							}
							region_file.write_bytes(&region_bytes)?;
							MemorySnapshotStorage::File(region_file)
						},
						None => MemorySnapshotStorage::Memory(region_bytes)
					}
				));
			}
		}

		// Return snapshot.
		Ok(MemorySnapshot {
			name: snapshot_name.to_string(),
			regions: snapshot_regions
		})
	}
}


pub trait MemorySnapshotSource<AddressType:AddressSourceType> {
	fn as_raw_address_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>>;
}
impl<AddressType:AddressSourceType> MemorySnapshotSource<AddressType> for &str {
	fn as_raw_address_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>> {
		self.to_string().as_raw_address_range(pmm)
	}
}
impl<AddressType:AddressSourceType> MemorySnapshotSource<AddressType> for String {
	fn as_raw_address_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>> {
		let module_info = pmm.get_module_info(self)?;
		Ok(vec![module_info.base_address()..module_info.base_address() + module_info.size()])
	}
}
impl<AddressType:AddressSourceType> MemorySnapshotSource<AddressType> for Range<AddressType> {
	fn as_raw_address_range(&self, _pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>> {
		Ok(vec![self.clone()])
	}
}
impl<AddressType:AddressSourceType> MemorySnapshotSource<AddressType> for MemoryRegion<AddressType> {
	fn as_raw_address_range(&self, pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>> {
		vec![self.clone()].as_raw_address_range(pmm)
	}
}
impl<AddressType:AddressSourceType> MemorySnapshotSource<AddressType> for Vec<MemoryRegion<AddressType>> {
	fn as_raw_address_range(&self, _pmm:&mut ProcessMemoryManipulator<AddressType>) -> Result<Vec<Range<AddressType>>, Box<dyn Error>> {
		let mut results:Vec<Range<AddressType>> = Vec::with_capacity(self.len());
		for region in self {
			if region.is_readable() {
				results.push(region.base_address()..region.base_address() + region.size());
			}
		}
		Ok(results)
	}
}