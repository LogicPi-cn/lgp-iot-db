use chrono::{Local, NaiveDateTime};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use taos::*;

#[derive(Serialize, Deserialize)]
pub struct AdxlData {
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

impl AdxlData {
    pub fn _random() -> Self {
        let mut rng = rand::thread_rng();
        let naive = Local::now().timestamp_millis();
        AdxlData {
            ts: NaiveDateTime::from_timestamp_millis(naive).unwrap(),
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

pub async fn init_tdengine_adxl(database_url: &str) -> anyhow::Result<Taos, taos::Error> {
    let taos = TaosBuilder::from_dsn(database_url)?.build().await?;
    taos.create_database("adxl").await?;
    taos.use_database("adxl").await?;
    taos::sync::Queryable::exec(
        &taos,
        "CREATE STABLE if NOT EXISTS adxl355 (
    ts        TIMESTAMP ,
    device_id INT       ,
    x         FLOAT     ,
    y         FLOAT     , 
    z         FLOAT     ,
    xf        FLOAT     ,
    yf        FLOAT     ,
    zf        FLOAT     ,
    ax        FLOAT     ,
    ay        FLOAT     ,
    az        FLOAT     ,
    axf       FLOAT     ,
    ayf       FLOAT     ,
    azf       FLOAT     ,
    t         FLOAT     ,
    tf        FLOAT     ,
    bat       FLOAT     )
    TAGS     (groupId INT)
    ",
    )?;

    Ok(taos)
}

pub async fn insert_adxl(new_data: AdxlData, taos: &Taos) -> Result<usize, Error> {
    let mut stmt = Stmt::init(&taos)?;
    stmt.prepare("INSERT INTO ? USING adxl355 TAGS(?) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")?;

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
        ColumnView::from_floats(vec![new_data.xf]),
        ColumnView::from_floats(vec![new_data.yf]),
        ColumnView::from_floats(vec![new_data.zf]),
        ColumnView::from_floats(vec![new_data.ax]),
        ColumnView::from_floats(vec![new_data.ay]),
        ColumnView::from_floats(vec![new_data.az]),
        ColumnView::from_floats(vec![new_data.axf]),
        ColumnView::from_floats(vec![new_data.ayf]),
        ColumnView::from_floats(vec![new_data.azf]),
        ColumnView::from_floats(vec![new_data.t]),
        ColumnView::from_floats(vec![new_data.tf]),
        ColumnView::from_floats(vec![new_data.bat]),
    ];

    stmt.bind(&values)?;
    stmt.add_batch()?;
    // execute.
    let rows = stmt.execute()?;

    Ok(rows)
}

pub async fn query_adxl(device_id: i32, taos: &Taos) -> Vec<AdxlData> {
    let mut datas = Vec::new();

    let mut result = taos
        .query("SELECT ts, x,y,z FROM adxl355 LIMIT 1")
        .await
        .unwrap();

    // print column names
    let meta = result.fields();
    println!("{}", meta.iter().map(|field| field.name()).join("\t"));

    // print rows
    let rows = result.rows();
    for row in rows {
        let row = row?;
        for (_name, value) in row {
            print!("{}\t", value);
        }
        println!();
    }

    datas
}

#[cfg(test)]
mod tests {}
