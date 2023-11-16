use std::env;

use chrono::{Duration, Local};
use lgp_iot_db::models::adxl_data_v2::{init_tdengine_adxl, query_adxl_by_date};
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_APP_LOG", "info");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    let taos = init_tdengine_adxl("taos://db.21up.cn:6030", "adxl").await?;

    // query records in last 30mins
    let now = Local::now();
    let last = now - Duration::minutes(30);

    let records =
        query_adxl_by_date(&taos, 9999, last.timestamp_millis(), now.timestamp_millis()).await;

    for record in records {
        println!("{}", record);
    }

    Ok(())
}
