#[cfg(test)]
mod tests {
	use crate::{ MemorySnapshot, ProcessMemoryManipulator, ProcessMemoryManipulator64, active_process_name };
	use file_ref::FileRef;
use mini_rand::RandomNumber;



	#[test]
	fn test_memory_snapshot_memory() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let random_tag:[u16; 100] = (0..100).map(|_| u16::random()).collect::<Vec<u16>>().try_into().unwrap();
		let random_tag_address:u64 = &random_tag as *const [u16; 100] as u64;
		let random_tag_bytes:Vec<u8> = random_tag.iter().map(|value| value.to_le_bytes()).flatten().collect();
		let mut random_tag_found:bool = false;

		let memory_snapshot:MemorySnapshot<u64> = pmm.create_memory_snapshot("test_name", None).unwrap();
		for (region, data_source) in memory_snapshot.regions() {
			if region.base_address() < random_tag_address && region.base_address() + region.size() > random_tag_address {
				let offset:u64 = random_tag_address - region.base_address();
				if let Ok(bytes) = data_source.get_bytes() {
					assert_eq!(&bytes[offset as usize..offset as usize + random_tag_bytes.len()], random_tag_bytes);
					random_tag_found = true;
				}
			}
		}
		assert!(random_tag_found, "Random tag was not found in memory snapshot.");
	}

	#[test]
	fn test_memory_snapshot_file() {
		let process_name:String = active_process_name();
		let mut pmm:ProcessMemoryManipulator<u64> = ProcessMemoryManipulator64::new(&process_name, false);

		let cache_dir:FileRef = FileRef::new("target/snapshot_cache");
		if cache_dir.exists() {
			cache_dir.delete().unwrap();
		}

		let random_tag:[u16; 100] = (0..100).map(|_| u16::random()).collect::<Vec<u16>>().try_into().unwrap();
		let random_tag_address:u64 = &random_tag as *const [u16; 100] as u64;
		let random_tag_bytes:Vec<u8> = random_tag.iter().map(|value| value.to_le_bytes()).flatten().collect();
		let mut random_tag_found:bool = false;

		let memory_snapshot:MemorySnapshot<u64> = pmm.create_memory_snapshot("test_name", Some(cache_dir.path())).unwrap();
		for (region, data_source) in memory_snapshot.regions() {
			if region.base_address() < random_tag_address && region.base_address() + region.size() > random_tag_address {
				let offset:u64 = random_tag_address - region.base_address();
				if let Ok(bytes) = data_source.get_bytes() {
					assert_eq!(&bytes[offset as usize..offset as usize + random_tag_bytes.len()], random_tag_bytes);
					random_tag_found = true;
				}
			}
		}

		assert!(cache_dir.list_files_recurse().len() > 10, "Cache dir should contain more files.");
		assert!(random_tag_found, "Random tag was not found in memory snapshot.");
		drop(memory_snapshot);
		assert!(!cache_dir.exists(), "Cache dir should be removed after snapshot is dropped from memory.");
	}
}