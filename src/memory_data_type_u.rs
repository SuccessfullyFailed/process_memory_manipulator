#[cfg(test)]
mod tests {
	use mini_rand::RandomNumber;
	use crate::MemoryDataType;
	use std::fmt::Debug;



	fn test_conversion_value<T:MemoryDataType + RandomNumber + PartialEq + Clone + Debug>() {
		const RANDOM_NUMBER_QUANTITY:usize = 500;
		
		for _ in 0..RANDOM_NUMBER_QUANTITY {
			let value:T = T::random();

			let as_bytes_be:Vec<u8> = value.clone().mdt_to_be_bytes();
			assert_eq!(value, T::mdt_from_be_bytes(as_bytes_be));

			let as_bytes_le:Vec<u8> = value.clone().mdt_to_le_bytes();
			assert_eq!(value, T::mdt_from_le_bytes(as_bytes_le));
		}
	}



	#[test]
	fn test_u8_conversion() { test_conversion_value::<u8>(); }
	#[test]
	fn test_i8_conversion() { test_conversion_value::<i8>(); }
	#[test]
	fn test_u16_conversion() { test_conversion_value::<u16>(); }
	#[test]
	fn test_i16_conversion() { test_conversion_value::<i16>(); }
	#[test]
	fn test_u32_conversion() { test_conversion_value::<u32>(); }
	#[test]
	fn test_i32_conversion() { test_conversion_value::<i32>(); }
	#[test]
	fn test_u64_conversion() { test_conversion_value::<u64>(); }
	#[test]
	fn test_i64_conversion() { test_conversion_value::<i64>(); }
	
	#[test]
	fn test_f32_conversion() { test_conversion_value::<f32>(); }
	#[test]
	fn test_f64_conversion() { test_conversion_value::<f64>(); }
}