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
	let player_position:[f32; 3] = [96.5, 108.4, 912.80];
	let player:Player = Player { _health: 100, position: &player_position };
	let player_reference:&Player = &player;

	// Print addresses of required variables.
	println!("{:?}", &read_test_var as *const u32);
	println!("{:?}", &write_test_var_left as *const u16);
	println!("{:?}", &write_test_var_right as *const u16);
	println!("{:?}", &write_test_success as *const u8);
	println!("{:?}", &player_reference as *const &Player);
	println!("{:#x}", unsafe { (&player.position as *const &[f32; 3] as *const u8).offset_from(&player as *const Player as *const u8) });
	println!("{:#x}", unsafe { (&player.position[2] as *const f32 as *const u8).offset_from(&player_position as *const [f32; 3] as *const u8) });
	println!("0x666"); // Last print tag.

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



struct Player<'a> {
	_health:u64,
	position:&'a [f32; 3]
}