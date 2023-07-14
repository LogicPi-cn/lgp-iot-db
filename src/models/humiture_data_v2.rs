use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use crc::{Crc, CRC_8_MAXIM_DOW};
use log::{debug, error, warn};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use taos::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct HumitureData {
    pub ts: DateTime<Local>, // Time Stamp from device
    pub sn: i32,             // Device Serial Number
    pub device_id: i64,      // Device Unique ID
    pub group_id: i32,       // Group id
    pub type_id: i32,        // Type
    pub temperature: f32,
    pub humidity: f32,
}

// print
impl fmt::Display for HumitureData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "HumitureData {{ sn: 0x{:08X}, id: 0x{:016X}, group: 0x{:02X}, type: 0x{:02X}, ts: {}, t: {}℃, h: {}% }}",
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

impl HumitureData {
    pub fn new(sn: i32, device_id: i64, group_id: i32, type_id: i32, t: f32, h: f32) -> Self {
        HumitureData {
            ts: Local::now(),
            sn,
            device_id,
            group_id,
            type_id,
            temperature: t,
            humidity: h,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        HumitureData {
            ts: Local::now(),
            sn: 0x00000001,
            device_id: 0x0000111122223333,
            group_id: 0,
            type_id: 0,
            temperature: rng.gen_range(-20.0..50.0),
            humidity: rng.gen_range(1.0..100.0),
        }
    }

    // generate a sin/cos wave for test
    pub fn test_wave(r: f32, angle: f32) -> Self {
        HumitureData {
            sn: 0x00000002,
            ts: Local::now(),
            device_id: 0x0000111122223333,
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
        bytes.extend_from_slice(&self.device_id.to_be_bytes());
        // sn
        bytes.extend_from_slice(&self.sn.to_be_bytes());
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
        let temperature_x10 = (self.temperature * 10.0) as i16;
        let humidity_x10 = (self.humidity * 10.0) as i16;
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
                    let device_id = i64::from_be_bytes(bytes[3..11].try_into().unwrap());
                    let sn = i32::from_be_bytes(bytes[11..15].try_into().unwrap());
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
                    let now = Local::now();

                    for i in 0..n {
                        let tt = i16::from_be_bytes([
                            bytes[(23 + i * 2) as usize],
                            bytes[(24 + i * 2) as usize],
                        ]);
                        let hh = i16::from_be_bytes([
                            bytes[(23 + i * 2 + 2 * n) as usize],
                            bytes[(24 + i * 2 + 2 * n) as usize],
                        ]);
                        let t = tt as f32 / 10.0;
                        let h = hh as f32 / 10.0;

                        // seperate the time(1h/2h/4h) into n slices
                        let ts = now - Duration::minutes((((n - i) * interval) as i32).into());

                        let new_data = HumitureData {
                            sn,
                            ts,
                            device_id,
                            group_id,
                            type_id,
                            temperature: t,
                            humidity: h,
                        };

                        // temperature and humidity check
                        if t > 100.0 || t < -40.0 || h > 100.0 || h < 0.0 {
                            // check if test data
                            if group_id == 0 && type_id == 0 {
                                debug!("{}", new_data);
                                result.push(new_data);
                            } else {
                                warn!("{} --- Overflow!", new_data)
                            }
                        } else {
                            debug!("{}", new_data);
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

impl HumitureData {}

pub async fn init_tdengine_humiture(database_url: &str, db_name: &str) -> Result<Taos, Error> {
    let taos = TaosBuilder::from_dsn(database_url)?.build().await?;
    taos.create_database(db_name).await?;
    taos.use_database(db_name).await?;
    taos.exec(
        "CREATE STABLE if NOT EXISTS humiture (
    ts          TIMESTAMP,
    sn          INT      ,
    device_id   BIGINT   ,
    group_id    INT      ,
    type_id     INT      ,
    temperature FLOAT    ,
    humidity    FLOAT    )
    TAGS     (groupId INT)
    ",
    )
    .await?;

    Ok(taos)
}

pub async fn insert_humiture(new_data: HumitureData, taos: &Taos) -> Result<usize, Error> {
    let mut stmt = Stmt::init(&taos)?;
    stmt.prepare("INSERT INTO ? USING humiture TAGS(?) VALUES(?, ?, ?, ?, ?, ?, ?)")?;

    // bind table name and tags
    stmt.set_tbname_tags(
        format!("g{:06}", new_data.group_id),
        &[taos::Value::Int(new_data.group_id)],
    )?;

    // bind values.
    let values = vec![
        ColumnView::from_millis_timestamp(vec![new_data.ts.timestamp_millis()]),
        ColumnView::from_ints(vec![new_data.sn]),
        ColumnView::from_big_ints(vec![new_data.device_id]),
        ColumnView::from_ints(vec![new_data.group_id]),
        ColumnView::from_ints(vec![new_data.type_id]),
        ColumnView::from_floats(vec![new_data.temperature]),
        ColumnView::from_floats(vec![new_data.humidity]),
    ];

    stmt.bind(&values)?;
    stmt.add_batch()?;
    // execute.
    let rows = stmt.execute()?;

    debug!("Inserted {} rows", rows);

    Ok(rows)
}

pub async fn query_humiture(taos: &Taos, group_id: i32, n: i32) -> Vec<HumitureData> {
    let sql = format!(
        "SELECT * FROM humiture.{} LIMIT {}",
        format!("g{:06}", group_id),
        n
    );

    let mut result = taos.query(sql).await.unwrap();

    let mut records = Vec::new();

    match result.deserialize().try_collect().await {
        Ok(nrecords) => records = nrecords,
        Err(e) => {
            error!("query error: {:?}", e);
        }
    }

    records
}

#[cfg(test)]
mod tests {

    use crate::models::humiture_data_v2::HumitureData;
    use std::{env, sync::Once};

    static INIT: Once = Once::new();

    pub fn init() {
        INIT.call_once(|| {
            env::set_var("RUST_APP_LOG", "debug");
            pretty_env_logger::init_custom_env("RUST_APP_LOG");
        });
    }

    #[test]
    fn test_to_bytes() {
        init();

        let bytes =
            HumitureData::new(0x00000001, 0x0000111122223333, 0, 0, -20.5, -10.5).to_bytes();
        let result = HumitureData::from_bytes(&bytes, 1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_from_bytes() {
        use hex;

        init();

        let hex_string = "5aa546e85f0022005700aa6c6f6769bd0217051e0d1105010d010e010e001f0110011001ff011000ffffffffff010d02af02af02ae024f027b027a02ff028102ffffffffff02ae34006d";
        let bytes = hex::decode(hex_string).unwrap();

        let result = HumitureData::from_bytes(&bytes, 12);

        assert_eq!(result.len(), 10);
    }
}
