use chrono::{Duration, Local, NaiveDateTime};
use diesel::{AsChangeset, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl};
use log::{debug, error, info};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::{errors::PkgError, schema::humiture_datas, DbError};

#[derive(Serialize, Deserialize, Queryable, Debug, AsChangeset)]
pub struct HumitureData {
    pub id: i32,
    pub sn: String,        // Device Serial Number
    pub device_id: String, // Device Unique ID
    pub ts: NaiveDateTime, // Time Stamp from device
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = humiture_datas)]
pub struct NewHumitureData {
    pub sn: String,        // Device Serial Number
    pub ts: NaiveDateTime, // Time Stamp fro device
    pub device_id: String, // Device Unique ID
    pub temperature: f32,
    pub humidity: f32,
}

impl NewHumitureData {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        info!("ts: {}", naive);

        NewHumitureData {
            sn: String::from("00000001"),
            ts: NaiveDateTime::parse_from_str(&naive, fmt).unwrap(),
            device_id: String::from("test"),
            temperature: rng.gen_range(-20.0..50.0),
            humidity: rng.gen_range(1.0..100.0),
        }
    }

    // generate a sin/cos wave for test
    pub fn test_wave(r: f32, angle: f32) -> Self {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        info!("ts: {}", naive);

        NewHumitureData {
            sn: String::from("00000002"),
            ts: NaiveDateTime::parse_from_str(&naive, fmt).unwrap(),
            device_id: String::from("test_wave"),
            temperature: r * (angle * 3.1415926 / 180.0).sin(),
            humidity: r * (angle * 3.1415926 / 180.0).cos(),
        }
    }

    // get some data from bytes
    pub fn from_bytes(bytes: &[u8], n: i32) -> Vec<Self> {
        let mut result = Vec::new();

        // unpackage the data
        if bytes.len() > 0 {
            // compare the head
            if bytes[0..2] == [0x5A, 0xA5] {
                // check the length
                let len = bytes[2] as usize;
                if len == bytes.len() - 4 {
                    let id = &bytes[3..11];
                    let sn = &bytes[11..15];

                    // get current time
                    let fmt = "%Y-%m-%d %H:%M:%S";
                    let naive = Local::now().format(fmt).to_string();
                    let now = NaiveDateTime::parse_from_str(&naive, fmt).unwrap();

                    for i in 0..n {
                        let tt = u16::from_be_bytes([
                            bytes[(23 + i * 2) as usize],
                            bytes[(24 + i * 2) as usize],
                        ]);
                        let hh = u16::from_be_bytes([
                            bytes[(23 + i * 2 + 2 * n) as usize],
                            bytes[(24 + i * 2 + 2 * n) as usize],
                        ]);
                        let t = tt as f32 / 10.0;
                        let h = hh as f32 / 10.0;

                        // seperate the time(2h) into n slices
                        let ts = now - Duration::minutes(((n - i) * (120 / n)).into());

                        debug!("id={:?}", hex::encode(id));
                        debug!("sn={:?}", hex::encode(sn));
                        debug!("t={}, h={}", t, h);
                        debug!("ts={}", ts);

                        // add the vector
                        result.push(NewHumitureData {
                            sn: hex::encode(sn),
                            ts,
                            device_id: hex::encode(id),
                            temperature: t,
                            humidity: h,
                        })
                    }
                } else {
                    error!(
                        "length error : real is {}, total len is {}",
                        len,
                        bytes.len()
                    );
                }
            } else {
                error!("bad header");
            }
        }

        return result;
    }

    // get new data from bytes string
    pub fn from_string(bytes: &str) -> Result<Self, PkgError> {
        let mut t = 0.0;
        let mut h = 0.0;
        let mut sn = "";
        let mut id = "";

        // unpackage the data
        if bytes.len() > 0 {
            // compare the head
            let head = "5aa5";
            if bytes.get(..4) == head.get(..4) {
                id = bytes.get(6..22).unwrap();
                sn = bytes.get(22..30).unwrap();
                let tt = u16::from_str_radix(bytes.get(46..50).unwrap(), 16).unwrap();
                let hh = u16::from_str_radix(bytes.get(50..54).unwrap(), 16).unwrap();
                t = tt as f32 / 10.0;
                h = hh as f32 / 10.0;

                // info!("id={}", id);
                // info!("sn={}", sn);
                // info!("t={}, h={}", t, h);
            } else {
                return Err(PkgError::new(
                    String::from("pkg"),
                    String::from("bad package head"),
                ));
            }
        }

        // get current time
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        let now = NaiveDateTime::parse_from_str(&naive, fmt).unwrap();

        Ok(NewHumitureData {
            sn: sn.to_string(),
            ts: now,
            device_id: id.to_string(),
            temperature: t,
            humidity: h,
        })
    }
}

impl HumitureData {
    pub fn all(conn: &mut PgConnection) -> Result<Vec<HumitureData>, DbError> {
        let items = humiture_datas::table.load::<Self>(conn)?;
        Ok(items)
    }

    pub fn find(id: i32, conn: &mut PgConnection) -> Result<HumitureData, DbError> {
        let result = humiture_datas::table.find(id).first::<HumitureData>(conn)?;
        Ok(result)
    }

    pub fn create(data: NewHumitureData, conn: &mut PgConnection) -> Result<HumitureData, DbError> {
        let result = diesel::insert_into(humiture_datas::table)
            .values(&data)
            .get_result(conn)
            .expect("Error on Create");
        Ok(result)
    }

    pub fn delete(id: i32, conn: &mut PgConnection) -> Result<usize, DbError> {
        let num_deleted = diesel::delete(humiture_datas::table.find(id))
            .execute(conn)
            .expect("Error on Delete");
        Ok(num_deleted)
    }
}

#[test]
fn test_from_bytes() {
    use env_logger;
    use hex;

    env_logger::init();

    let hex_string = "5aa546e85f0022005700aa6c6f6769bd0217051e0d1105010d010e010e001f0110011001ff011000ffffffffff010d02af02af02ae024f027b027a02ff028102ffffffffff02ae34006d";
    let bytes = hex::decode(hex_string).unwrap();

    let result = NewHumitureData::from_bytes(&bytes, 12);

    assert_eq!(result.len(), 12);
}
