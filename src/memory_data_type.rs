pub trait MemoryDataType {
	const BYTES_SIZE:usize;

	fn mdt_from_be_bytes(bytes:Vec<u8>) -> Self;
	fn mdt_to_be_bytes(self) -> Vec<u8>;
	fn mdt_from_le_bytes(bytes:Vec<u8>) -> Self;
	fn mdt_to_le_bytes(self) -> Vec<u8>;
}
#[macro_export]
macro_rules! impl_data_type_for_atom {
	($type_name:ident, $bytes_size:expr) => {
		impl MemoryDataType for $type_name {
			const BYTES_SIZE:usize = core::mem::size_of::<$type_name>();
			
			fn mdt_from_be_bytes(bytes:Vec<u8>) -> Self {
				Self::from_be_bytes(bytes.try_into().unwrap())
			}
			fn mdt_to_be_bytes(self) -> Vec<u8> {
				self.to_be_bytes().to_vec()
			}
			fn mdt_from_le_bytes(bytes:Vec<u8>) -> Self {
				Self::from_le_bytes(bytes.try_into().unwrap())
			}
			fn mdt_to_le_bytes(self) -> Vec<u8> {
				self.to_le_bytes().to_vec()
			}
		}
	};
}
#[macro_export]
macro_rules! impl_data_type_for_array {
	($type_name:ident, $bytes_size:expr, $count:expr) => {
		impl MemoryDataType for [$type_name; $count] {
			const BYTES_SIZE:usize = core::mem::size_of::<$type_name>();

			fn mdt_from_be_bytes(bytes:Vec<u8>) -> Self {
				bytes.chunks(4).map(|chunk| <$type_name>::mdt_from_be_bytes(chunk.try_into().unwrap())).collect::<Vec<$type_name>>().try_into().unwrap()
			}
			fn mdt_to_be_bytes(self) -> Vec<u8> {
				self.map(|value| value.mdt_to_be_bytes()).into_iter().flatten().collect::<Vec<u8>>().try_into().unwrap()
			}
			fn mdt_from_le_bytes(bytes:Vec<u8>) -> Self {
				bytes.chunks(4).map(|chunk| <$type_name>::mdt_from_le_bytes(chunk.try_into().unwrap())).collect::<Vec<$type_name>>().try_into().unwrap()
			}
			fn mdt_to_le_bytes(self) -> Vec<u8> {
				self.map(|value| value.mdt_to_le_bytes()).into_iter().flatten().collect::<Vec<u8>>().try_into().unwrap()
			}
		}
	};
}