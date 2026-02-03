use std::ops::{ BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign };


#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MemoryAccessToken(pub(crate) u32);
impl MemoryAccessToken {

	// All available access tokens.
	pub const NONE:MemoryAccessToken = MemoryAccessToken(0);
	pub const READ_CONTROL:MemoryAccessToken = MemoryAccessToken(0x20000);
	pub const WRITE_DAC:MemoryAccessToken = MemoryAccessToken(0x40000);
	pub const WRITE_OWNER:MemoryAccessToken = MemoryAccessToken(0x80000);
	pub const SYNCHRONIZE:MemoryAccessToken = MemoryAccessToken(0x100000);
	pub const DELETE:MemoryAccessToken = MemoryAccessToken(0x10000);
	pub const PROCESS_STANDARD_ACCESS:MemoryAccessToken = MemoryAccessToken(0xF0000);
	pub const PROCESS_CREATE_PROCESS:MemoryAccessToken = MemoryAccessToken(0x80);
	pub const PROCESS_CREATE_THREAD:MemoryAccessToken = MemoryAccessToken(0x2);
	pub const PROCESS_DUP_HANDLE:MemoryAccessToken = MemoryAccessToken(0x40);
	pub const PROCESS_QUERY_INFORMATION:MemoryAccessToken = MemoryAccessToken(0x400);
	pub const PROCESS_QUERY_LIMITED_INFORMATION:MemoryAccessToken = MemoryAccessToken(0x1000);
	pub const PROCESS_SET_INFORMATION:MemoryAccessToken = MemoryAccessToken(0x200);
	pub const PROCESS_SET_QUOTA:MemoryAccessToken = MemoryAccessToken(0x100);
	pub const PROCESS_SUSPEND_RESUME:MemoryAccessToken = MemoryAccessToken(0x800);
	pub const PROCESS_TERMINATE:MemoryAccessToken = MemoryAccessToken(0x1);
	pub const PROCESS_VM_OPERATION:MemoryAccessToken = MemoryAccessToken(0x8);
	pub const PROCESS_VM_READ:MemoryAccessToken = MemoryAccessToken(0x10);
	pub const PROCESS_VM_WRITE:MemoryAccessToken = MemoryAccessToken(0x20);
	pub const AS_LIST:&[(&str, MemoryAccessToken)] = &[
		("READ_CONTROL", MemoryAccessToken::READ_CONTROL),
		("WRITE_DAC", MemoryAccessToken::WRITE_DAC),
		("WRITE_OWNER", MemoryAccessToken::WRITE_OWNER),
		("SYNCHRONIZE", MemoryAccessToken::SYNCHRONIZE),
		("DELETE", MemoryAccessToken::DELETE),
		("PROCESS_STANDARD_ACCESS", MemoryAccessToken::PROCESS_STANDARD_ACCESS),
		("PROCESS_CREATE_PROCESS", MemoryAccessToken::PROCESS_CREATE_PROCESS),
		("PROCESS_CREATE_THREAD", MemoryAccessToken::PROCESS_CREATE_THREAD),
		("PROCESS_DUP_HANDLE", MemoryAccessToken::PROCESS_DUP_HANDLE),
		("PROCESS_QUERY_INFORMATION", MemoryAccessToken::PROCESS_QUERY_INFORMATION),
		("PROCESS_QUERY_LIMITED_INFORMATION", MemoryAccessToken::PROCESS_QUERY_LIMITED_INFORMATION),
		("PROCESS_SET_INFORMATION", MemoryAccessToken::PROCESS_SET_INFORMATION),
		("PROCESS_SET_QUOTA", MemoryAccessToken::PROCESS_SET_QUOTA),
		("PROCESS_SUSPEND_RESUME", MemoryAccessToken::PROCESS_SUSPEND_RESUME),
		("PROCESS_TERMINATE", MemoryAccessToken::PROCESS_TERMINATE),
		("PROCESS_VM_OPERATION", MemoryAccessToken::PROCESS_VM_OPERATION),
		("PROCESS_VM_READ", MemoryAccessToken::PROCESS_VM_READ),
		("PROCESS_VM_WRITE", MemoryAccessToken::PROCESS_VM_WRITE)
	];
}
impl Default for MemoryAccessToken {
	fn default() -> Self {
		MemoryAccessToken::NONE
	}
}
impl BitAnd<MemoryAccessToken> for MemoryAccessToken {
	type Output = MemoryAccessToken;
	fn bitand(self, rhs:MemoryAccessToken) -> Self::Output {
		MemoryAccessToken(self.0 & rhs.0)
	}
}
impl BitAndAssign<MemoryAccessToken> for MemoryAccessToken {
	fn bitand_assign(&mut self, rhs:MemoryAccessToken) {
		self.0 &= rhs.0;
	}
}
impl BitOr<MemoryAccessToken> for MemoryAccessToken {
	type Output = MemoryAccessToken;
	fn bitor(self, rhs:MemoryAccessToken) -> Self::Output {
		MemoryAccessToken(self.0 | rhs.0)
	}
}
impl BitOrAssign<MemoryAccessToken> for MemoryAccessToken {
	fn bitor_assign(&mut self, rhs:MemoryAccessToken) {
		self.0 |= rhs.0;
	}
}
impl BitXor<MemoryAccessToken> for MemoryAccessToken {
	type Output = MemoryAccessToken;
	fn bitxor(self, rhs:MemoryAccessToken) -> Self::Output {
		MemoryAccessToken(self.0 ^ rhs.0)
	}
}
impl BitXorAssign<MemoryAccessToken> for MemoryAccessToken {
	fn bitxor_assign(&mut self, rhs:MemoryAccessToken) {
		self.0 ^= rhs.0;
	}
}