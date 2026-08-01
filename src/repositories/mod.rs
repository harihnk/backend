use crate::models::{EventRow,UserEvent};
use clickhouse::Client;

/// Initialize the ClickHouse database and events table
pub async fn init_db(client: &Client) -> Result<(), clickhouse::error::Error> {
    // Create the analytics database
    client
        .query("CREATE DATABASE IF NOT EXISTS analytics")
        .execute()
        .await?;

    // Create the events table
    client
        .query(
            "CREATE TABLE IF NOT EXISTS analytics.events (
            user_id: String,
            session_id: String,
            device :String,
            browser :String,
            os :String,
            country :String,
            region :String,
            city :String, 
            latitude :Float64,        
            longitude :Float64,
            click_count :UInt32,
            user_journey :String,
            screen_width :UInt32,
            screen_height :UInt32    
            ) ENGINE = MergeTree()
             ORDER BY (session_id, user_id)"
        )
        .execute()
        .await?;

    Ok(())
}

/// Batch insert enriched events into ClickHouse
pub async fn insert_events(
    client: &Client,
    events: Vec<EventRow>,
) -> Result<(), clickhouse::error::Error> {
    let mut inserter = client.insert("analytics.events")?;

    for event in events {
        inserter.write(&event).await?;
    }

    inserter.end().await?;

    Ok(())
}
pub async fn get_events(
    client: &Client,
) -> Result<Vec<UserEvent>, clickhouse::error::Error> {
    let events = client
    .query("SELECT 
     user_id,
     session_id,
     device,
     browser,
     os,
     country,
     region,
     city,
     latitude,
     longitude,
     click_count,
     user_journey,
     screen_width,
     screen_height
    FROM analytics.events")
    .fetch_all::<UserEvent>()
    .await?;
    Ok(events)
}
    
