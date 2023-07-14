use lgp_iot_db::models::humiture_data_v2::{
    init_tdengine_humiture, insert_humiture, query_humiture, HumitureData,
};
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let taos = init_tdengine_humiture("taos://localhost:6030", "humiture").await?;

    let random_data = HumitureData::random();
    insert_humiture(random_data, &taos).await?;

    let records = query_humiture(&taos, 0, 10).await;
    for record in records {
        println!("{}", record);
    }

    Ok(())
}
