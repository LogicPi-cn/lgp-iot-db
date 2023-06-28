use lgp_iot_db::models::adxl_data_v2::{init_tdengine_adxl, insert_adxl, query_adxl, AdxlData};
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let taos = init_tdengine_adxl("taos://30.30.30.242:6030").await?;

    let random_data = AdxlData::_random();
    insert_adxl(random_data, &taos).await?;

    let _ = query_adxl(9999, &taos).await;

    Ok(())
}
