use std::fmt;

use chrono::{DateTime, Local};
use log::error;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use taos::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdxlData {
    pub device_id: i32,
    pub ts: DateTime<Local>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub t: f32,
    pub bat: f32,
}

// print
impl fmt::Display for AdxlData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AdxlData {{ id: {}, ts: {}, x: {}, y: {}, z: {}, t: {}℃, bat: {}% }}",
            self.device_id, self.ts, self.x, self.y, self.z, self.t, self.bat
        )
    }
}

impl AdxlData {
    pub fn _random() -> Self {
        let mut rng = rand::thread_rng();
        AdxlData {
            ts: Local::now(),
            device_id: 9999,
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(-1.0..1.0),
            t: rng.gen_range(-40.0..100.0),
            bat: rng.gen_range(1.0..100.0),
        }
    }
    // generate a sin/cos wave for test
    pub fn test_wave(r: f32, angle: f32) -> Self {
        AdxlData {
            ts: Local::now(),
            device_id: 9999,
            x: r * (angle * 3.1415926 / 180.0).sin(),
            y: r * ((angle + 90.0) * 3.1415926 / 180.0).sin(),
            z: r * ((angle + 180.0) * 3.1415926 / 180.0).sin(),
            t: r * ((angle + 270.0) * 3.1415926 / 180.0).sin(),
            bat: 100.0,
        }
    }
}

pub async fn init_tdengine_adxl(
    database_url: &str,
    db_name: &str,
) -> anyhow::Result<Taos, taos::Error> {
    let taos = TaosBuilder::from_dsn(database_url)?.build().await?;
    taos.create_database(db_name).await?;
    taos.use_database(db_name).await?;
    taos::sync::Queryable::exec(
        &taos,
        "CREATE STABLE if NOT EXISTS adxl355 (
    ts        TIMESTAMP ,
    device_id INT       ,
    x         FLOAT     ,
    y         FLOAT     , 
    z         FLOAT     ,
    t         FLOAT     ,
    bat       FLOAT     )
    TAGS     (groupId INT)
    ",
    )?;

    Ok(taos)
}

pub async fn insert_adxl(new_data: AdxlData, taos: &Taos) -> Result<usize, Error> {
    let mut stmt = Stmt::init(&taos)?;
    stmt.prepare("INSERT INTO ? USING adxl355 TAGS(?) VALUES(?, ?, ?, ?, ?, ?, ?)")?;

    // bind table name and tags
    stmt.set_tbname_tags(
        format!("g{:06}", new_data.device_id),
        &[taos::Value::Int(new_data.device_id)],
    )?;

    // bind values.
    let values = vec![
        ColumnView::from_millis_timestamp(vec![new_data.ts.timestamp_millis()]),
        ColumnView::from_ints(vec![new_data.device_id]),
        ColumnView::from_floats(vec![new_data.x]),
        ColumnView::from_floats(vec![new_data.y]),
        ColumnView::from_floats(vec![new_data.z]),
        ColumnView::from_floats(vec![new_data.t]),
        ColumnView::from_floats(vec![new_data.bat]),
    ];

    stmt.bind(&values)?;
    stmt.add_batch()?;
    // execute.
    let rows = stmt.execute()?;

    Ok(rows)
}

pub async fn query_adxl(taos: &Taos, device_id: i32, n: i32) -> Vec<AdxlData> {
    let sql = format!(
        "SELECT * FROM adxl355.{} LIMIT {}",
        format!("g{:06}", device_id),
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
