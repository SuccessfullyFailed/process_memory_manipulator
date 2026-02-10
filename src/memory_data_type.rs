use std::fmt::Debug;



pub trait MemoryDataType {
	const BYTES_SIZE:usize;

	fn mdt_from_be_bytes(bytes:Vec<u8>) -> Self;
	fn mdt_to_be_bytes(self) -> Vec<u8>;
	fn mdt_from_le_bytes(bytes:Vec<u8>) -> Self;
	fn mdt_to_le_bytes(self) -> Vec<u8>;
}
pub trait MemoryDataTypeSized:MemoryDataType + Sized {
	fn mdt_flip_endian(self) -> Self {
		Self::mdt_from_le_bytes(self.mdt_to_be_bytes())
	}
}
impl<T:MemoryDataType + Sized> MemoryDataTypeSized for T {}


/* IMPLEMENT FOR ATOMS */

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
impl_data_type_for_atom!(u8,1);
impl_data_type_for_atom!(i8,1);
impl_data_type_for_atom!(u16,2);
impl_data_type_for_atom!(i16,2);
impl_data_type_for_atom!(u32,4);
impl_data_type_for_atom!(i32,4);
impl_data_type_for_atom!(u64,8);
impl_data_type_for_atom!(i64,8);
impl_data_type_for_atom!(u128,16);
impl_data_type_for_atom!(i128,16);
impl_data_type_for_atom!(f32,4);
impl_data_type_for_atom!(f64,8);



/* IMPLEMENT FOR LISTS */

impl<T:MemoryDataType + Debug, const ARRAY_SIZE:usize> MemoryDataType for [T; ARRAY_SIZE] {
	const BYTES_SIZE:usize = T::BYTES_SIZE * ARRAY_SIZE;

	fn mdt_from_be_bytes(bytes:Vec<u8>) -> Self {
		bytes.chunks(T::BYTES_SIZE).map(|chunk| T::mdt_from_be_bytes(chunk.try_into().unwrap())).collect::<Vec<T>>().try_into().unwrap()
	}
	fn mdt_to_be_bytes(self) -> Vec<u8> {
		self.map(|value| value.mdt_to_be_bytes()).into_iter().flatten().collect::<Vec<u8>>().try_into().unwrap()
	}
	fn mdt_from_le_bytes(bytes:Vec<u8>) -> Self {
		bytes.chunks(T::BYTES_SIZE).map(|chunk| T::mdt_from_le_bytes(chunk.try_into().unwrap())).collect::<Vec<T>>().try_into().unwrap()
	}
	fn mdt_to_le_bytes(self) -> Vec<u8> {
		self.map(|value| value.mdt_to_le_bytes()).into_iter().flatten().collect::<Vec<u8>>().try_into().unwrap()
	}
}