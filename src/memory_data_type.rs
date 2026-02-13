pub trait MemoryDataType:Sized + Copy {
	const BYTES_SIZE:usize;
	type Bytes:AsRef<[u8]> + AsMut<[u8]> + Copy;


	fn mdt_from_be_bytes(bytes:Self::Bytes) -> Self;
	fn mdt_to_be_bytes(self) -> Self::Bytes;
	fn mdt_to_be_bytes_vec(self) -> Vec<u8>;
	fn mdt_from_le_bytes(bytes:Self::Bytes) -> Self;
	fn mdt_to_le_bytes(self) -> Self::Bytes;
	fn mdt_to_le_bytes_vec(self) -> Vec<u8>;
	
	fn mdt_flip_endian(self) -> Self {
		Self::mdt_from_le_bytes(self.mdt_to_be_bytes())
	}
}


/* IMPLEMENT FOR ATOMS */

macro_rules! impl_data_type_for_atom {
	($type_name:ident, $bytes_size:expr) => {
		impl MemoryDataType for $type_name {
			const BYTES_SIZE:usize = core::mem::size_of::<$type_name>();
			type Bytes = [u8; core::mem::size_of::<$type_name>()];
			
			fn mdt_from_be_bytes(bytes:Self::Bytes) -> Self {
				Self::from_be_bytes(bytes)
			}
			fn mdt_to_be_bytes(self) -> Self::Bytes {
				self.to_be_bytes()
			}
			fn mdt_to_be_bytes_vec(self) -> Vec<u8> {
				self.mdt_to_be_bytes().to_vec()
			}
			fn mdt_from_le_bytes(bytes:Self::Bytes) -> Self {
				Self::from_le_bytes(bytes)
			}
			fn mdt_to_le_bytes(self) -> Self::Bytes {
				self.to_le_bytes()
			}
			fn mdt_to_le_bytes_vec(self) -> Vec<u8> {
				self.mdt_to_le_bytes().to_vec()
			}
		}

		impl<const LIST_SIZE:usize> MemoryDataType for [$type_name; LIST_SIZE] {
			const BYTES_SIZE:usize = { LIST_SIZE * core::mem::size_of::<$type_name>() };
			type Bytes = [u8; core::mem::size_of::<$type_name>()];
			
			fn mdt_from_be_bytes(bytes:Self::Bytes) -> Self {
				(0..LIST_SIZE).map(|index|
					<$type_name>::from_be_bytes(bytes[index * $bytes_size..(index + 1) * $bytes_size].try_into().unwrap())
				).collect::<Vec<$type_name>>().try_into().unwrap()
			}
			fn mdt_to_be_bytes(self) -> Self::Bytes {
				self.into_iter().map(|value| value.mdt_to_be_bytes()).flatten().collect::<Vec<u8>>().try_into().unwrap()
			}
			fn mdt_to_be_bytes_vec(self) -> Vec<u8> {
				self.mdt_to_be_bytes().to_vec()
			}
			fn mdt_from_le_bytes(bytes:Self::Bytes) -> Self {
				(0..LIST_SIZE).map(|index|
					<$type_name>::from_le_bytes(bytes[index * $bytes_size..(index + 1) * $bytes_size].try_into().unwrap())
				).collect::<Vec<$type_name>>().try_into().unwrap()
			}
			fn mdt_to_le_bytes(self) -> Self::Bytes {
				self.into_iter().map(|value| value.mdt_to_le_bytes()).flatten().collect::<Vec<u8>>().try_into().unwrap()
			}
			fn mdt_to_le_bytes_vec(self) -> Vec<u8> {
				self.mdt_to_le_bytes().to_vec()
			}
		}
	};
}
impl_data_type_for_atom!(u8, 1);
impl_data_type_for_atom!(i8, 1);
impl_data_type_for_atom!(u16, 2);
impl_data_type_for_atom!(i16, 2);
impl_data_type_for_atom!(u32, 4);
impl_data_type_for_atom!(i32, 4);
impl_data_type_for_atom!(u64, 8);
impl_data_type_for_atom!(i64, 8);
impl_data_type_for_atom!(u128, 16);
impl_data_type_for_atom!(i128, 16);
impl_data_type_for_atom!(f32, 4);
impl_data_type_for_atom!(f64, 8);