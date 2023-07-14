use std::env;

use lgp_iot_db::models::adxl_data_v2::{init_tdengine_adxl, insert_adxl, query_adxl, AdxlData};
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_APP_LOG", "debug");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    let taos = init_tdengine_adxl("taos://localhost:6030", "adxl355").await?;

    let random_data = AdxlData::_random();
    insert_adxl(random_data, &taos).await?;

    let records = query_adxl(&taos, 9999, 10).await;

    for record in records {
        println!("{}", record);
    }

    Ok(())
}
