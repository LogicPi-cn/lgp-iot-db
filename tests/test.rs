use lgp_iot_db::models::humiture_datas::{average, NewHumitureData};

use std::{env, sync::Once};

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        env::set_var("RUST_APP_LOG", "debug");
        pretty_env_logger::init_custom_env("RUST_APP_LOG");
    });
}

#[test]
fn test_from_bytes() {
    use hex;

    init();

    let hex_string = "5aa546e85f0022005700aa6c6f6769bd0217051e0d1105010d010e010e001f0110011001ff011000ffffffffff010d02af02af02ae024f027b027a02ff028102ffffffffff02ae34006d";
    let bytes = hex::decode(hex_string).unwrap();

    let result = NewHumitureData::from_bytes(&bytes, 12);

    assert_eq!(result.len(), 10);
}

#[test]
fn test_average() {
    init();

    let avg = average(25.5, 25.7, 35.0, 2.0);

    assert_eq!(avg, 25.6);
}

#[test]
fn test_to_bytes() {
    init();

    let bytes = NewHumitureData::new("00000000", "0000111122223333", 0, 0, -20.5, -10.5).to_bytes();
    let result = NewHumitureData::from_bytes(&bytes, 1);
    assert_eq!(result.len(), 1);
}
