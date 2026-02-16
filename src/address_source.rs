use std::{ fmt::{ Debug, Display, LowerHex }, ops::{ Add, AddAssign, Sub, SubAssign } };
use winapi::ctypes::c_void;
use crate::MemoryDataType;



pub trait AddressSourceType:Send + Sync + Debug + Display + Default + LowerHex + Copy + PartialEq + PartialOrd + Add<Output=Self> + AddAssign + Sub<Output=Self> + SubAssign + MemoryDataType {
	fn to_usize(&self) -> usize;
	fn to_c_void_ptr(&self) -> *const c_void;
	fn to_c_void_ptr_mut(&self) -> *mut c_void;
	fn from_u64(address:u64) -> Self;
	fn from_usize(address:usize) -> Self;
	fn wrapping_sub(self, address:Self) -> Self;
	fn max_relative_jmp_offset() -> Self;
}
impl AddressSourceType for u64 {
	fn to_usize(&self) -> usize {
		*self as usize
	}
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
	fn from_u64(address:u64) -> Self {
		address
	}
	fn from_usize(address:usize) -> Self {
		address as u64
	}
	fn wrapping_sub(self, address:Self) -> Self {
		u64::wrapping_sub(self, address)
	}
	fn max_relative_jmp_offset() -> Self {
		const BYTES:u64 = i32::MAX as u64;
		BYTES
	}
}
impl AddressSourceType for u32 {
	fn to_usize(&self) -> usize {
		*self as usize
	}
	fn to_c_void_ptr(&self) -> *const c_void {
		*self as *const c_void
	}
	fn to_c_void_ptr_mut(&self) -> *mut c_void {
		*self as *mut c_void
	}
	fn from_u64(address:u64) -> Self {
		address as u32
	}
	fn from_usize(address:usize) -> Self {
		address as u32
	}
	fn wrapping_sub(self, address:Self) -> Self {
		u32::wrapping_sub(self, address)
	}
	fn max_relative_jmp_offset() -> Self {
		const BYTES:u32 = i32::MAX as u32;
		BYTES
	}
}