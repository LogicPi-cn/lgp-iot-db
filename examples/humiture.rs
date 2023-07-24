use chrono::{Duration, Local};
use lgp_iot_db::models::humiture_data_v2::{
    init_tdengine_humiture, query_humiture_by_date, query_humiture_by_group, query_humiture_by_sn,
};
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let taos = init_tdengine_humiture("taos://db.21up.cn:6030", "humiture").await?;

    // let random_data = HumitureData::random();
    // insert_humiture(random_data, &taos).await?;

    let now = Local::now();
    let last = now - Duration::minutes(30);

    let records = query_humiture_by_date(
        &taos,
        0x0000111122223333,
        last.timestamp_millis(),
        now.timestamp_millis(),
    )
    .await;

    for record in records {
        println!("{}", record);
    }

    let records = query_humiture_by_group(&taos, 0, 30).await;
    for record in records {
        println!("{}", record);
    }

    let records = query_humiture_by_sn(&taos, 2, 10).await;

    for record in records {
        println!("{}", record);
    }

    Ok(())
}
