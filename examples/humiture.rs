use lgp_iot_db::models::humiture_data_v2::{init_tdengine_humiture, insert_humiture, HumitureData};
use taos::*;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let taos = init_tdengine_humiture("taos://localhost:6030", "humiture_test").await?;

    let random_data = HumitureData::random();
    insert_humiture(random_data, &taos).await?;

    sleep(std::time::Duration::from_millis(100)).await;

    let random_data = HumitureData::random();
    insert_humiture(random_data, &taos).await?;

    sleep(std::time::Duration::from_millis(100)).await;

    let random_data = HumitureData::random();
    insert_humiture(random_data, &taos).await?;

    sleep(std::time::Duration::from_millis(100)).await;

    let random_data = HumitureData::random();
    insert_humiture(random_data, &taos).await?;

    Ok(())
}
