#[allow(unused_assignments)]
fn main() {
	use std::{ thread::sleep, time::{ Duration, Instant } };
	
	// Get a basic interval to wait for test results.
	const INTERVAL:Duration = Duration::from_millis(1);
	let program_timeout:Instant = Instant::now() + Duration::from_secs(5);

	// Create values for testing.
	let read_test_var:u32 = 1618;
	let write_test_var_left:u16 = 0x80;
	let write_test_var_right:u16 = 0;
	let mut write_test_success:u8 = 0;

	// Print addresses of required variables.
	println!("{:?}", &read_test_var as *const u32);
	println!("{:?}", &write_test_var_left as *const u16);
	println!("{:?}", &write_test_var_right as *const u16);
	println!("{:?}", &write_test_success as *const u8);

	// Wait for write test to change value.
	while Instant::now() < program_timeout && write_test_var_left != write_test_var_right {
		sleep(INTERVAL);
	}
	write_test_success = 1;

	// Sleep until tests have processed.
	while Instant::now() < program_timeout {
		sleep(INTERVAL);
	}
}