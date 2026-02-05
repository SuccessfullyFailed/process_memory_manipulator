use winapi::{ shared::{minwindef::{ DWORD, HINSTANCE__ }, ntdef::HANDLE as ModuleHandle }, um::{ psapi::{ EnumProcessModulesEx, GetModuleBaseNameA, GetModuleInformation, MODULEINFO }, winnt::HANDLE as WinHandle } };
use crate::{ AddressSourceType, MemoryAccessToken, ProcessMemoryManipulator };
use std::{ error::Error, mem, ptr };



pub struct ModuleInfo<AddressType:AddressSourceType> {
	name:String,
	base_address:AddressType,
	size:AddressType,
	entry_point:AddressType
}
impl<AddressType:AddressSourceType> ModuleInfo<AddressType> {

	/* PROPERTY GETTER METHODS */

	/// Get the name of the module.
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Get the base address of the module.
	pub fn base_address(&self) -> AddressType {
		self.base_address
	}

	/// Get the size of the module in bytes.
	pub fn size(&self) -> AddressType {
		self.size
	}

	/// Get the entry-point of the module.
	pub fn entry_point(&self) -> AddressType {
		self.entry_point
	}
}



impl<AddressType:AddressSourceType> ProcessMemoryManipulator<AddressType> {
	
	/// Get main process base address.
	pub fn get_base_address(&mut self) -> Result<AddressType, Box<dyn Error>> {
		self.get_base_module_info().map(|info| info.base_address)
	}
	
	/// Get main process module info.
	pub fn get_base_module_info(&mut self) -> Result<ModuleInfo<AddressType>, Box<dyn Error>> {
		let process_name:String = self.process_name().to_string();
		self.get_module_info(&process_name)
	}
	
	/// Get base address of specific module.
	pub fn get_module_base_address(&mut self, module_name:&str) -> Result<AddressType, Box<dyn Error>> {
		self.get_module_info(module_name).map(|info| info.base_address)
	}
	
	/// Get a modules info.
	pub fn get_module_info(&mut self, module_name:&str) -> Result<ModuleInfo<AddressType>, Box<dyn Error>> {
		const MODULE_INFO_ACCESS:MemoryAccessToken = MemoryAccessToken(MemoryAccessToken::PROCESS_QUERY_LIMITED_INFORMATION.0 | MemoryAccessToken::PROCESS_VM_READ.0);
		unsafe {

			// Get module handles.
			let process_handle:WinHandle = self.win_handle(MODULE_INFO_ACCESS)?;
			let mut module_handles_buffer:Vec<*mut HINSTANCE__> = vec![ptr::null_mut(); 1024 * 10];
			let mut bytes_returned:u32 = 0; 
			if EnumProcessModulesEx(process_handle, module_handles_buffer.as_mut_ptr(), (module_handles_buffer.len() * size_of::<ModuleHandle>()) as DWORD, &mut bytes_returned, 0x03) != 0 {
				
				// Loop through modules.
				let modules_count:usize = bytes_returned as usize / size_of::<ModuleHandle>();
				for module_handle in module_handles_buffer.iter().take(modules_count) {

					// Get module name.
					let loop_module_name_buf:[u8; 256] = [0u8; 256];
					if GetModuleBaseNameA(process_handle, *module_handle, loop_module_name_buf.as_ptr() as *mut _, loop_module_name_buf.len() as DWORD) == 0 {
						continue;
					}
					let loop_module_name:String = String::from_utf8_lossy(&loop_module_name_buf.into_iter().take_while(|char| *char != 0).collect::<Vec<u8>>()).to_string();

					// Check match module name matches target module name.
					if loop_module_name.contains(module_name) {

						let mut module_info:MODULEINFO = mem::zeroed();
						if GetModuleInformation(process_handle, *module_handle, &mut module_info, size_of::<MODULEINFO>() as u32) == 0 {
							continue;
						}

						return Ok(ModuleInfo {
							name: loop_module_name,
							base_address: AddressType::from_u64(module_info.lpBaseOfDll as u64),
							size: AddressType::from_u64(module_info.SizeOfImage as u64),
							entry_point: AddressType::from_u64(module_info.EntryPoint as u64)
						});
					}
				}
			}
		}


		// Return the base address or None.
		Err(format!("Could not get module information of module '{module_name}'.").into())
	}
}