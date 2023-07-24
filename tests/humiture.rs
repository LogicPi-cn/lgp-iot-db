#[cfg(test)]
mod test_humiture {

    use chrono::{Duration, Local};
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

        let hex_string = "5aa546e85f0022005700aa6c6f6769bd0217051e0d1105010d010e010e001f0110011001ff011000ffffffffff010d02af02af02ae024f027b027a02ff028102ffffffffff02ae34006d";
        let bytes = hex::decode(hex_string).unwrap();

        let result = HumitureData::from_bytes(&bytes, 12);

        assert_eq!(result.len(), 10);
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
