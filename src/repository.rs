use clickhouse::Client;
use crate::errors::AppError;
use crate::models::*;
#[derive(Debug, clickhouse::Row, serde::Deserialize)]
struct CountResult {
    cnt: u64,
}
#[derive(Clone)]
pub struct EventRepository {
    client: Client,
}

impl EventRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn insert_events(&self, rows: Vec<ClickEventRow>) -> Result<(), AppError> {
        let mut insert = self.client.insert("click_events")?;
        for row in rows {
            insert.write(&row).await?;
        }
        insert.end().await?;
        Ok(())
    }

    pub async fn get_heatmap(
        &self,
        page_url: &str,
        event_type: &str,
    ) -> Result<Vec<HeatmapPoint>, AppError> {
        let query = r#"
            SELECT x_percent, y_percent, count() as hits
            FROM click_events
            WHERE page_url = ? AND event_type = ?
              AND x_percent IS NOT NULL AND y_percent IS NOT NULL
            GROUP BY x_percent, y_percent
            ORDER BY hits DESC
        "#;

        self.client
            .query(query)
            .bind(page_url)
            .bind(event_type)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_navigation_flow(
        &self,
        user_id: &str,
    ) -> Result<Vec<NavigationStep>, AppError> {
        let query = r#"
            SELECT session_id, seq, previous_page, current_page, external_referrer, timestamp
            FROM click_events
            WHERE user_id = ? AND event_type = 'pageview'
            ORDER BY timestamp, seq
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_navigation_stats(&self) -> Result<Vec<NavigationStats>, AppError> {
        let query = r#"
            SELECT previous_page, current_page, count() as transitions
            FROM click_events
            WHERE event_type = 'pageview'
            GROUP BY previous_page, current_page
            ORDER BY transitions DESC
        "#;

        self.client
            .query(query)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_journey(
        &self,
        user_id: &str,
    ) -> Result<Vec<NavigationStep>, AppError> {
        let query = r#"
            SELECT session_id, seq, previous_page, current_page, external_referrer, timestamp
            FROM click_events
            WHERE user_id = ?
            ORDER BY timestamp, seq
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_click_logs(&self) -> Result<Vec<ClickLog>, AppError> {
        let query = r#"
            SELECT page_url, element_id, element_text, count() as clicks
            FROM click_events
            WHERE event_type = 'click'
            GROUP BY page_url, element_id, element_text
            ORDER BY clicks DESC
        "#;

        self.client
            .query(query)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_click_count(
        &self,
        user_id: &str,
    ) -> Result<Vec<ClickCountResult>, AppError> {
        let query = r#"
            SELECT user_id, session_id, sum(click_count) as total_clicks
            FROM click_events
            WHERE user_id = ? AND event_type = 'click'
            GROUP BY user_id, session_id
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_funnel_counts(
        &self,
        steps: &[String],
    ) -> Result<Vec<(String, u64)>, AppError> {
        let mut results = Vec::new();

        for step in steps {
            let query = r#"
                SELECT count(DISTINCT session_id) as cnt
                FROM click_events
                WHERE (funnel_step_name = ? OR section = ? OR current_page = ?)
                  AND event_type IN ('funnel', 'section_view', 'pageview')
            "#;

            let count: CountResult = self.client
                .query(query)
                .bind(step)
                .bind(step)
                .bind(step)
                .fetch_one()
                .await?;

            results.push((step.clone(), count.cnt));
        }

        Ok(results)
    }

    pub async fn get_scroll_depth(
        &self,
        page_url: &str,
    ) -> Result<Vec<ScrollDepthResult>, AppError> {
        let query = r#"
            SELECT page_url, avg(scroll_depth) as avg_depth
            FROM click_events
            WHERE page_url = ? AND scroll_depth IS NOT NULL
            GROUP BY page_url
        "#;

        self.client
            .query(query)
            .bind(page_url)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_countries(&self) -> Result<Vec<LocationStats>, AppError> {
        let query = r#"
            SELECT country as location, count(DISTINCT user_id) as visitors
            FROM click_events
            WHERE country != '' AND country != 'Private' AND country != 'Unknown'
            GROUP BY country
            ORDER BY visitors DESC
        "#;

        self.client
            .query(query)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_regions(&self, country: &str) -> Result<Vec<LocationStats>, AppError> {
        let query = r#"
            SELECT region as location, count(DISTINCT user_id) as visitors
            FROM click_events
            WHERE country = ? AND region != '' AND region != 'Private' AND region != 'Unknown'
            GROUP BY region
            ORDER BY visitors DESC
        "#;

        self.client
            .query(query)
            .bind(country)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_cities(&self, region: &str) -> Result<Vec<LocationStats>, AppError> {
        let query = r#"
            SELECT city as location, count(DISTINCT user_id) as visitors
            FROM click_events
            WHERE region = ? AND city != '' AND city != 'Private' AND city != 'Unknown'
            GROUP BY city
            ORDER BY visitors DESC
        "#;

        self.client
            .query(query)
            .bind(region)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_metadata(&self, user_id: &str) -> Result<Option<UserMetadataRow>, AppError> {
        let query = r#"
            SELECT country, region, city, browser, os, device_type
            FROM click_events
            WHERE user_id = ? AND country != '' AND country != 'Private' AND country != 'Unknown'
            ORDER BY timestamp DESC
            LIMIT 1
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_optional()
            .await
            .map_err(Into::into)
    }

    pub async fn get_navigation_flow_detailed(&self, user_id: &str) -> Result<Vec<NavigationFlowStep>, AppError> {
        let query = r#"
            SELECT session_id, seq, event_type, previous_page, current_page, hash, section, timestamp
            FROM click_events
            WHERE user_id = ? AND event_type IN ('pageview', 'hashchange', 'section_view')
            ORDER BY timestamp ASC, seq ASC
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_clicks_detailed(&self, user_id: &str) -> Result<Vec<UserClickDetail>, AppError> {
        let query = r#"
            SELECT previous_page, current_page, viewport_width, viewport_height, element_tag, element_id, element_text, x_pos, y_pos, x_percent, y_percent, timestamp
            FROM click_events
            WHERE user_id = ? AND event_type = 'click'
            ORDER BY timestamp ASC, seq ASC
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_all()
            .await
            .map_err(Into::into)
    }

    pub async fn get_user_exit_way(&self, user_id: &str) -> Result<Option<ExitWayDetail>, AppError> {
        let query = r#"
            SELECT element_id AS exit_type, element_text AS target_url, timestamp
            FROM click_events
            WHERE user_id = ? AND event_type = 'exit'
            ORDER BY timestamp DESC
            LIMIT 1
        "#;

        self.client
            .query(query)
            .bind(user_id)
            .fetch_optional()
            .await
            .map_err(Into::into)
    }
}