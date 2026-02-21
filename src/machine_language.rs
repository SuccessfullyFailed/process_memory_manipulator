use crate::{ AddressSourceType, MemoryDataType };
use std::ops::{ Add, Mul, Range };



const RELATIVE_JMP_SIZE:usize = 5;



#[derive(Clone, PartialEq)]
enum MachineCodeInner<AddressType:AddressSourceType> {
	RawBytes(Vec<u8>),
	Variables(Vec<Vec<u8>>),
	JmpOffset(i32),
	JmpOver(Box<MachineCode<AddressType>>),
	JmpTo(AddressType),
	Combined(Vec<MachineCode<AddressType>>),
	Repeat((Box<MachineCode<AddressType>>, usize))
}
#[derive(Clone, PartialEq)]
pub struct MachineCode<AddressType:AddressSourceType>(MachineCodeInner<AddressType>);
impl<AddressType:AddressSourceType> MachineCode<AddressType> {

	/* CONSTRUCTOR METHODS */

	/// Create a list of raw bytes.
	pub fn raw_bytes(bytes:Vec<u8>) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::RawBytes(bytes))
	}

	/// Create a list of bytes that do nothing for the given byte-count.
	pub fn do_nothing(bytes_count:usize) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::RawBytes(vec![0x90; bytes_count]))
	}

	/// Create a space for a variable. Does not automatically jump over it.
	pub fn variable<ValueType:MemoryDataType>(value:ValueType) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::Variables(vec![value.mdt_to_le_bytes_vec()]))
	}

	/// Jump a relative offset. The offset is the amount of bytes to skip after this command, so starting from the byte after the jump command has completed.
	pub fn jmp_offset(offset:i32) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::JmpOffset(offset))
	}

	/// Jump over and write the given bytes.
	pub fn jmp_over(inner:MachineCode<AddressType>) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::JmpOver(Box::new(inner)))
	}

	/// Jump to an absolute position.
	pub fn jmp_to(address:AddressType) -> MachineCode<AddressType> {
		MachineCode(MachineCodeInner::JmpTo(address))
	}



	/* USAGE METHODS */

	/// Convert the code to bytes and fill the list with do-nothing bytes until the target amount is reached. Does not insert it into the instruction address, but uses that for jump and offset references.
	pub fn to_bytes_amount(self, instruction_address:Option<AddressType>, big_endian:bool, target_bytes_count:usize) -> Vec<u8> {
		let mut bytes:Vec<u8> = self.to_bytes(instruction_address, big_endian);
		let original_byte_count:usize = bytes.len();
		if original_byte_count < target_bytes_count {
			bytes.extend(vec![0x90; target_bytes_count - original_byte_count]);
		}
		bytes
	}

	/// Convert the code to bytes. Does not insert it into the instruction address, but uses that for jump and offset references.
	pub fn to_bytes(self, instruction_address:Option<AddressType>, big_endian:bool) -> Vec<u8> {
		let address_to_bytes:fn(AddressType) -> Vec<u8> = if big_endian { MemoryDataType::mdt_to_be_bytes_vec } else { MemoryDataType::mdt_to_le_bytes_vec };
		let combine:fn(Vec<u8>, Vec<u8>) -> Vec<u8> = |left:Vec<u8>, right:Vec<u8>| vec![left, right].into_iter().flatten().collect();
		match self.0 {
			MachineCodeInner::RawBytes(bytes) => {
				bytes
			},

			MachineCodeInner::JmpOffset(offset) => {
				combine(vec![0xE9], if big_endian { offset.mdt_to_be_bytes_vec() } else { offset.mdt_to_le_bytes_vec() })
			},

			MachineCodeInner::JmpOver(inner) => {
				let inner_bytes:Vec<u8> = inner.to_bytes(instruction_address.map(|address| address + AddressType::from_usize(RELATIVE_JMP_SIZE)), big_endian);
				combine(MachineCode::jmp_offset(inner_bytes.len() as i32).to_bytes(instruction_address, big_endian), inner_bytes)
			},

			MachineCodeInner::Variables(mut bytes_per_variable) => {
				if big_endian {
					bytes_per_variable.iter_mut().for_each(|bytes| bytes.reverse());
				}
				bytes_per_variable.into_iter().flatten().collect()
			},

			MachineCodeInner::JmpTo(target_address) => {
				const JUMP_BYTE:u8 = 0xFF;
				const QWORD_BYTE:u8 = 0x25;
				match instruction_address {
					Some(start_address) => {
						let jmp_offset_abs:AddressType = if target_address > start_address { target_address - start_address } else { start_address - target_address };
						if jmp_offset_abs < AddressType::max_relative_jmp_offset() {
							MachineCode::jmp_offset(target_address.to_i32() - start_address.to_i32() - RELATIVE_JMP_SIZE as i32).to_bytes(Some(start_address), big_endian)
						} else {
							MachineCode::jmp_to(target_address).to_bytes(None, big_endian)
						}
					},
					None => {
						if AddressType::BYTES_SIZE == 4 {
							combine(vec![JUMP_BYTE, QWORD_BYTE], address_to_bytes(target_address))
						} else {
							combine(vec![JUMP_BYTE, QWORD_BYTE, 0x00, 0x00, 0x00, 0x00], target_address.mdt_to_le_bytes_vec())
						}
					}
				}
			},

			MachineCodeInner::Combined(machine_codes) => {
				let mut start_address:Option<AddressType> = instruction_address;
				let mut output_bytes:Vec<u8> = Vec::new();
				for machine_code in machine_codes {
					let additional_bytes: Vec<u8> = machine_code.to_bytes(start_address, big_endian);
					start_address = start_address.map(|value| value + AddressType::from_usize(additional_bytes.len()));
					output_bytes.extend(additional_bytes);
				}
				output_bytes
			},

			MachineCodeInner::Repeat((machine_code, repeat_count)) => {
				let mut start_address:Option<AddressType> = instruction_address;
				let mut output_bytes:Vec<u8> = Vec::new();
				for _ in 0..repeat_count {
					let additional_bytes: Vec<u8> = machine_code.clone().to_bytes(start_address, big_endian);
					start_address = start_address.map(|value| value + AddressType::from_usize(additional_bytes.len()));
					output_bytes.extend(additional_bytes);
				}
				output_bytes
			}
		}
	}

	/// Get the estimated length of the amount of bytes when converting. As size can depend on multiple factors, this will return a minimum and maximum amount.
	pub fn estimated_byte_count(&self) -> Range<usize> {
		match &self.0 {
			MachineCodeInner::RawBytes(bytes) => {
				let byte_count:usize = bytes.len();
				byte_count..byte_count
			},

			MachineCodeInner::Variables(bytes_per_variable) => {
				let flat_size:usize = bytes_per_variable.iter().map(|variable| variable.len()).sum::<usize>();
				flat_size..flat_size
			},

			MachineCodeInner::JmpOffset(_offset) => {
				RELATIVE_JMP_SIZE..RELATIVE_JMP_SIZE
			},

			MachineCodeInner::JmpOver(inner) => {
				let inner_range:Range<usize> = inner.estimated_byte_count();
				inner_range.start + RELATIVE_JMP_SIZE..inner_range.end + RELATIVE_JMP_SIZE
			},

			MachineCodeInner::JmpTo(target_address) => {
				let max:usize = {
					if target_address.mdt_to_le_bytes_vec().len() == 4 {
						6
					} else {
						14
					}
				};
				RELATIVE_JMP_SIZE..max
			},

			MachineCodeInner::Combined(machine_codes) => {
				let mut min:usize = 0;
				let mut max:usize = 0;
				for machine_code in machine_codes {
					let range:Range<usize> = machine_code.estimated_byte_count();
					min += range.start;
					max += range.end;
				}
				min..max
			},

			MachineCodeInner::Repeat((machine_code, repeat_count)) => {
				let raw_range = machine_code.estimated_byte_count();
				raw_range.start * repeat_count..raw_range.end * repeat_count
			}
		}
	}
}
impl<AddressType:AddressSourceType> Add<MachineCode<AddressType>> for MachineCode<AddressType> {
	type Output = MachineCode<AddressType>;

	fn add(mut self, rhs:MachineCode<AddressType>) -> Self::Output {
		if let MachineCodeInner::Combined(list) = &mut self.0 {
			list.push(rhs);
			return self;
		}
		if let MachineCodeInner::Variables(variables) = &mut self.0 {
			if let MachineCodeInner::Variables(additional_variables) = rhs.0 {
				variables.extend(additional_variables);
				return self;
			}
		}
		MachineCode(MachineCodeInner::Combined(vec![self, rhs]))
	}
}
impl<AddressType:AddressSourceType> Add<Vec<u8>> for MachineCode<AddressType> {
	type Output = MachineCode<AddressType>;

	fn add(self, rhs:Vec<u8>) -> Self::Output {
		self + MachineCode::raw_bytes(rhs)
	}
}
impl<AddressType:AddressSourceType> Mul<usize> for MachineCode<AddressType> {
	type Output = MachineCode<AddressType>;

	fn mul(self, rhs:usize) -> Self::Output {
		MachineCode(MachineCodeInner::Repeat((Box::new(self), rhs)))
	}
}