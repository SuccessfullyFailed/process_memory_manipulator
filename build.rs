use std::error::Error;
use file_ref::FileRef;



const GENERATED_DATATYPE_FILE:FileRef = FileRef::new_const("src/memory_data_type_generated_impls.rs");
const GENERATED_DATATYPE_MIN_SIZE:u64 = 255;
const GENERATED_DATATYPE_ARRAY_LENGTH:usize = 8;





fn main() -> Result<(), Box<dyn Error>> {
	const DATA_TYPES:&[(&str, usize)] = &[("u8", 1), ("i8", 1), ("u16", 2), ("i16", 2), ("u32", 4), ("i32", 4), ("u64", 8), ("i64", 8), ("u128", 16), ("i128", 16), ("f32", 4), ("f64", 8)];

	if GENERATED_DATATYPE_FILE.exists() && GENERATED_DATATYPE_FILE.bytes_size() < GENERATED_DATATYPE_MIN_SIZE {
		GENERATED_DATATYPE_FILE.write(
			format!(
				"use crate::{{ MemoryDataType, impl_data_type_for_atom as iat, impl_data_type_for_array as iar }};\n{}",
				DATA_TYPES.iter().map(|(data_type, size)| 
					format!(
						"iat!({data_type},{size});{}",
						(0..GENERATED_DATATYPE_ARRAY_LENGTH).map(|list_length| format!("iar!({data_type},{size},{list_length});")).collect::<Vec<String>>().join("")
					)
				).collect::<Vec<String>>().join("\n")
			)
		)?;
	}

	Ok(())
}