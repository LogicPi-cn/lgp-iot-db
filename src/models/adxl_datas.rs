use chrono::{Local, NaiveDateTime};
use diesel::{AsChangeset, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl};
use log::info;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::{schema::adxl_datas, DbError};

#[derive(Serialize, Deserialize, Queryable, Debug, AsChangeset)]
pub struct AdxlData {
    pub id: i32,
    pub device_id: i32,
    pub ts: NaiveDateTime,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub xf: f32,
    pub yf: f32,
    pub zf: f32,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub axf: f32,
    pub ayf: f32,
    pub azf: f32,
    pub t: f32,
    pub tf: f32,
    pub bat: f32,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = adxl_datas)]
pub struct NewAdxlData {
    pub device_id: i32,
    pub ts: NaiveDateTime,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub xf: f32,
    pub yf: f32,
    pub zf: f32,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub axf: f32,
    pub ayf: f32,
    pub azf: f32,
    pub t: f32,
    pub tf: f32,
    pub bat: f32,
}

impl NewAdxlData {
    pub fn _random() -> Self {
        let mut rng = rand::thread_rng();
        let fmt = "%Y-%m-%d %H:%M:%S";
        let naive = Local::now().format(fmt).to_string();
        info!("ts: {}", naive);
        NewAdxlData {
            ts: NaiveDateTime::parse_from_str(&naive, fmt).unwrap(),
            device_id: 9999,
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(-1.0..1.0),
            xf: rng.gen_range(-1.0..1.0),
            yf: rng.gen_range(-1.0..1.0),
            zf: rng.gen_range(-1.0..1.0),
            ax: rng.gen_range(-90.0..90.0),
            ay: rng.gen_range(-1.0..1.0),
            az: rng.gen_range(-1.0..1.0),
            axf: rng.gen_range(-1.0..1.0),
            ayf: rng.gen_range(-1.0..1.0),
            azf: rng.gen_range(-1.0..1.0),
            t: rng.gen_range(-40.0..100.0),
            tf: rng.gen_range(-40.0..100.0),
            bat: rng.gen_range(1.0..100.0),
        }
    }
}

impl AdxlData {
    pub fn all(conn: &mut PgConnection) -> Result<Vec<AdxlData>, DbError> {
        let items = adxl_datas::table.load::<AdxlData>(conn)?;
        Ok(items)
    }

    pub fn find(id: i32, conn: &mut PgConnection) -> Result<AdxlData, DbError> {
        let result = adxl_datas::table.find(id).first::<AdxlData>(conn)?;
        Ok(result)
    }

    pub fn create(data: NewAdxlData, conn: &mut PgConnection) -> Result<AdxlData, DbError> {
        let result = diesel::insert_into(adxl_datas::table)
            .values(&data)
            .get_result(conn)
            .expect("Error on Create");
        Ok(result)
    }

    pub fn delete(id: i32, conn: &mut PgConnection) -> Result<usize, DbError> {
        let num_deleted = diesel::delete(adxl_datas::table.find(id))
            .execute(conn)
            .expect("Error on Delete");
        Ok(num_deleted)
    }
}
