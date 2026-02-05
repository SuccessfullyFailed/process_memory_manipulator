use crate::{ AddressSourceType, MemoryAccessToken, MemoryRegion, ProcessMemoryManipulator };
use mini_rand::RandomNumber;
use std::error::Error;
use file_ref::FileRef;



const CACHE_FILE_EXTENSION:&str = "pmmssc";



pub struct MemorySnapshot<AddressType:AddressSourceType> {
	name:String,
	regions:Vec<(MemoryRegion<AddressType>, MemorySnapshotSource)>
}
impl<AddressType:AddressSourceType> MemorySnapshot<AddressType> {

	/* PROPERTY GETTER METHODS */

	/// Get the name of the snapshot.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the regions of the snapshot.
	pub fn regions(&self) -> &[(MemoryRegion<AddressType>, MemorySnapshotSource)] {
		&self.regions
	}
}



pub enum MemorySnapshotSource {
	Memory(Vec<u8>),
	File(FileRef)
}
impl MemorySnapshotSource {

	/// Get the bytes of this source.
	pub fn get_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
		match self {
			MemorySnapshotSource::Memory(bytes) => Ok(bytes.clone()),
			MemorySnapshotSource::File(file) => file.read_bytes()
		}
	}
}
impl Drop for MemorySnapshotSource {
	fn drop(&mut self) {
		match self {
			MemorySnapshotSource::Memory(_) => {},
			MemorySnapshotSource::File(file) => {
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
		let cache_dir:Option<FileRef> = cache_dir_path.map(|path| FileRef::new(path));

		// Find memory regions.
		self.open_handle(SNAPSHOT_CREATION_ACCESS)?;
		let memory_regions:Vec<MemoryRegion<AddressType>> = self.memory_regions()?;

		// Read and store memory regions.
		let mut snapshot_regions:Vec<(MemoryRegion<AddressType>, MemorySnapshotSource)> = Vec::with_capacity(memory_regions.len());
		for region in memory_regions {
			if region.is_readable() {
				if let Ok(region_bytes) = self.read_bytes(region.base_address(), region.size().to_usize()) {
					snapshot_regions.push((
						region,
						match &cache_dir {
							Some(cache_dir) => {
								let mut region_file:FileRef = cache_dir.clone() + "/000." + CACHE_FILE_EXTENSION;
								while region_file.exists() {
									region_file = cache_dir.clone() + "/" + &u64::random().to_string() + "." + CACHE_FILE_EXTENSION;
								}
								region_file.write_bytes(&region_bytes)?;
								MemorySnapshotSource::File(region_file)
							},
							None => MemorySnapshotSource::Memory(region_bytes)
						}
					));
				}
			}
		}

		// Return snapshot.
		Ok(MemorySnapshot {
			name: snapshot_name.to_string(),
			regions: snapshot_regions
		})
	}
}