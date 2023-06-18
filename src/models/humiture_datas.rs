use chrono::{Datelike, Duration, Local, NaiveDateTime, Timelike};
use crc::{Crc, CRC_8_MAXIM_DOW};
use diesel::{AsChangeset, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl};
use log::{debug, error, warn};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::fmt;

use crate::{schema::humiture_datas, DbError};

// average
pub fn average(d0: f32, d1: f32, d2: f32, threshold: f32) -> f32 {
    let avg = (d0 + d1 + d2) / 3.0;
    let df1 = (d0 - avg).abs();
    let df2 = (d1 - avg).abs();
    let df3 = (d2 - avg).abs();

    debug!(
        "avg: {}, df1:{}, df2:{}, df3:{}, threshold: {}",
        avg, df1, df2, df3, threshold
    );

    if df1 <= threshold && df2 <= threshold && df3 <= threshold {
        avg
    } else {
        if df1 <= threshold && df2 <= threshold {
            (d0 + d1) / 2.0
        } else if df1 <= threshold && df3 <= threshold {
            (d0 + d2) / 2.0
        } else if df2 <= threshold && df3 <= threshold {
            (d1 + d2) / 2.0
        } else {
            average(d0, d1, d2, threshold + threshold)
        }
    }
}

// push hex str into vector
pub fn push_hex_str_into_vector(bytes: &mut Vec<u8>, sss: &str) {
    for i in (0..sss.len() - 2).step_by(2) {
        let hex_byte = u8::from_str_radix(&sss[i..i + 2], 16).unwrap();
        bytes.push(hex_byte);
    }
}

#[derive(Serialize, Deserialize, Queryable, Debug, AsChangeset)]
pub struct HumitureData {
    pub id: i32,
    pub sn: String,        // Device Serial Number
    pub device_id: String, // Device Unique ID
    pub group_id: i32,     // Group id
    pub type_id: i32,      // Type
    pub ts: NaiveDateTime, // Time Stamp from device
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = humiture_datas)]
pub struct NewHumitureData {
    pub sn: String,        // Device Serial Number
    pub device_id: String, // Device Unique ID
    pub group_id: i32,     // Group ID
    pub type_id: i32,      // Type
    pub ts: NaiveDateTime, // Time Stamp fro device
    pub temperature: f32,
    pub humidity: f32,
}

impl fmt::Display for NewHumitureData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HumitureData {{ sn: {}, id: {}, group:{}, type:{}, ts: {}, t: {}℃, h: {}% }}",
            self.sn,
            self.device_id,
            self.group_id,
            self.type_id,
            self.ts,
            self.temperature,
            self.humidity
        )
    }
}

impl NewHumitureData {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        // info!("ts: {}", naive);

        NewHumitureData {
            sn: String::from("00000001"),
            ts: NaiveDateTime::parse_from_str(&naive, fmt).unwrap(),
            device_id: String::from("0000111122223333"),
            group_id: 0,
            type_id: 0,
            temperature: rng.gen_range(-20.0..50.0),
            humidity: rng.gen_range(1.0..100.0),
        }
    }

    // generate a sin/cos wave for test
    pub fn test_wave(r: f32, angle: f32) -> Self {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        // info!("ts: {}", naive);

        NewHumitureData {
            sn: String::from("00000002"),
            ts: NaiveDateTime::parse_from_str(&naive, fmt).unwrap(),
            device_id: String::from("0000111122223333"),
            group_id: 0,
            type_id: 0,
            temperature: r * (angle * 3.1415926 / 180.0).sin(),
            humidity: r * (angle * 3.1415926 / 180.0).cos(),
        }
    }

    // convert to bytes
    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        // header
        bytes.push(0x5A);
        bytes.push(0xA5);
        // length
        bytes.push(26);
        // id
        bytes.extend_from_slice(&hex::decode(&self.device_id).unwrap());
        // sn
        bytes.extend_from_slice(&hex::decode(&self.sn).unwrap());
        // group id
        bytes.push(self.group_id as u8);

        // debug!("bytes length = {}", bytes.len());

        // type
        bytes.push(self.type_id as u8);

        // date
        let dt = self.ts;
        let hex_date = format!(
            "{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            dt.year() - 2000,
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        );
        bytes.extend_from_slice(&hex::decode(&hex_date).unwrap());

        // debug!("bytes length = {}", bytes.len());

        // temperature & humidity
        let temperature_x10 = (self.temperature * 10.0) as u16;
        let humidity_x10 = (self.humidity * 10.0) as u16;
        bytes.extend_from_slice(&temperature_x10.to_be_bytes());
        bytes.extend_from_slice(&humidity_x10.to_be_bytes());

        // battery
        bytes.push(0x63);
        // people
        bytes.push(0x00);

        // crc
        let crc8_checksum: Crc<u8> = Crc::<u8>::new(&CRC_8_MAXIM_DOW);
        let crc = crc8_checksum.checksum(&bytes[3..29]);

        bytes.push(crc);

        // debug!("bytes length = {}", bytes.len());

        bytes
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
                    let group_id = bytes[15] as i32;
                    let type_id = bytes[16] as i32;

                    // Time Interval
                    let interval = if len == 70 {
                        // 12 datas
                        ((bytes[72] as u8) >> 1) & 0x07
                    } else if len == 118 {
                        // 24 datas
                        ((bytes[120] as u8) >> 1) & 0x07
                    } else {
                        // single data
                        ((bytes[28] as u8) >> 1) & 0x07
                    };

                    // 000 -> 5min
                    // 001 -> 10min
                    // .........
                    // 111 -> 40min
                    let interval = ((interval + 1) * 5) as i32;

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

                        // seperate the time(1h/2h/4h) into n slices
                        let ts = now - Duration::minutes((((n - i) * interval) as i32).into());

                        // temperature and humidity check
                        if t > 100.0 || t < -40.0 || h > 100.0 || h < 0.0 {
                            warn!(
                                "Humiture Overflow! t:{}, h:{}, ts:{}, id:{}, sn:{}, group:{}, type:{}",
                                t,h,
                                ts.to_string(),
                                hex::encode(id),
                                hex::encode(sn),
                                group_id,
                                type_id
                            );
                        } else {
                            let new_data = NewHumitureData {
                                sn: hex::encode(sn),
                                ts,
                                device_id: hex::encode(id),
                                group_id,
                                type_id,
                                temperature: t,
                                humidity: h,
                            };

                            debug!("{}", new_data);

                            // add the vector
                            result.push(new_data);
                        }
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
