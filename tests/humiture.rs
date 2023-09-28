#[cfg(test)]
mod test_humiture {

    use chrono::{Duration, Local};
    use log::{debug, info};
    use std::{env, sync::Once};
    use tokio::test;

    use lgp_iot_db::models::humiture_data_v2::{
        init_tdengine_humiture, query_humiture_by_date, query_humiture_by_group,
        query_humiture_by_sn, HumitureData,
    };

    static INIT: Once = Once::new();

    pub fn init() {
        INIT.call_once(|| {
            env::set_var("RUST_APP_LOG", "info");
            pretty_env_logger::init_custom_env("RUST_APP_LOG");
        });
    }

    #[test]
    async fn test_to_bytes() {
        init();

        let bytes =
            HumitureData::new(0x00000001, 0x0000111122223333, 0, 0, -20.5, -10.5).to_bytes();
        let result = HumitureData::from_bytes(&bytes, 1);
        assert_eq!(result.len(), 1);
    }

    #[test]
    async fn test_from_bytes() {
        use hex;

        init();

        let hex_string = "5aa5767b9f770000005336000000380002150c0316020500be00be00c700c700be00c000c100c000c000c000c100c000bf00c000c200c200c400c300c400c300c200c300c200c203180320033803390331033203330333032b032d032f0325032d033003490352035b035b03510351034d034b035003526400da";
        let bytes = hex::decode(hex_string).unwrap();

        let result = HumitureData::from_bytes(&bytes, 24);

        for item in &result {
            info!("{:?}", item);
        }

        assert_eq!(result.len(), 24);
    }

    #[test]
    async fn test_query() {
        init();

        let taos = init_tdengine_humiture("taos://db.21up.cn:6030", "humiture")
            .await
            .unwrap();

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

        assert_eq!(records.len(), 149);

        let records = query_humiture_by_group(&taos, 0, 30).await;
        assert_eq!(records.len(), 30);

        let records = query_humiture_by_sn(&taos, 2, 10).await;
        assert_eq!(records.len(), 10);
    }
}
