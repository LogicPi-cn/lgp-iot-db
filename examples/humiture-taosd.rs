use lgp_iot_db::models::humiture_data_v2::HumitureData;
use taos::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let taos = TaosBuilder::from_dsn("taos://30.30.30.242:6030")?
        .build()
        .await?;
    taos.create_database("iot").await?;
    taos.use_database("iot").await?;
    taos.exec(
        "CREATE STABLE if NOT EXISTS humiture (
    ts          TIMESTAMP,       
    sn          INT, 
    device_id   BIGINT,  
    group_id    INT      ,        
    type_id     INT      ,        
    temperature FLOAT    ,        
    humidity    FLOAT   )          
    TAGS (location BINARY(64), groupId INT)
    ",
    )
    .await?;

    let mut stmt = Stmt::init(&taos)?;
    stmt.prepare("INSERT INTO ? USING humiture TAGS(?, ?) VALUES(?, ?, ?, ?, ?, ?, ?)")?;

    // bind table name and tags
    stmt.set_tbname_tags(
        "d1001",
        &[Value::VarChar("Nanjing.Jiangsu".into()), Value::Int(2)],
    )?;

    let random_data = HumitureData::random();

    // bind values.
    let values = vec![
        ColumnView::from_millis_timestamp(vec![random_data.ts.timestamp_millis()]),
        ColumnView::from_ints(vec![random_data.sn]),
        ColumnView::from_big_ints(vec![random_data.device_id]),
        ColumnView::from_ints(vec![random_data.group_id]),
        ColumnView::from_ints(vec![random_data.type_id]),
        ColumnView::from_floats(vec![random_data.temperature]),
        ColumnView::from_floats(vec![random_data.humidity]),
    ];

    stmt.bind(&values)?;

    let random_data = HumitureData::random();

    // bind values.
    let values2 = vec![
        ColumnView::from_millis_timestamp(vec![random_data.ts.timestamp_millis()]),
        ColumnView::from_ints(vec![random_data.sn]),
        ColumnView::from_big_ints(vec![random_data.device_id]),
        ColumnView::from_ints(vec![random_data.group_id]),
        ColumnView::from_ints(vec![random_data.type_id]),
        ColumnView::from_floats(vec![random_data.temperature]),
        ColumnView::from_floats(vec![random_data.humidity]),
    ];

    stmt.bind(&values2)?;

    stmt.add_batch()?;

    // execute.
    let rows = stmt.execute()?;
    assert_eq!(rows, 2);
    Ok(())
}
