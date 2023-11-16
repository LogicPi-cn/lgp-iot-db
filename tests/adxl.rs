#[cfg(test)]
mod test_adxl {

    use chrono::{Duration, Local};
    use lgp_iot_db::models::adxl_data_v2::{
        init_tdengine_adxl, insert_adxl, query_adxl_by_date, query_adxl_by_group, query_adxl_by_id,
        AdxlData,
    };
    use std::{env, sync::Once};
    use tokio::test;

    static INIT: Once = Once::new();

    pub fn init() {
        INIT.call_once(|| {
            env::set_var("RUST_APP_LOG", "info");
            pretty_env_logger::init_custom_env("RUST_APP_LOG");
        });
    }

    #[test]
    async fn test_insert() {
        init();
        let taos = init_tdengine_adxl("taos://localhost:6030", "adxl")
            .await
            .unwrap();

        let new_data = AdxlData::_random();
        let result = insert_adxl(new_data, &taos).await.unwrap();

        assert_eq!(result, 1);
    }

    #[test]
    async fn test_query() {
        init();
        let taos = init_tdengine_adxl("taos://db.21up.cn:6030", "adxl")
            .await
            .unwrap();

        // query  by date
        let now = Local::now();
        let last = now - Duration::minutes(30);
        let records =
            query_adxl_by_date(&taos, 9999, last.timestamp_millis(), now.timestamp_millis()).await;
        assert_eq!(records.len(), 60);

        // query by id
        let records = query_adxl_by_id(&taos, 9999, 10).await;
        assert_eq!(records.len(), 10);

        // query by group
        let records = query_adxl_by_group(&taos, 9999, 10).await;
        assert_eq!(records.len(), 10);
    }
}
