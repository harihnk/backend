use crate::errors::AppError;
use crate::models::*;
use crate::repository::EventRepository;

#[derive(Clone)]
pub struct EventService {
    repo: EventRepository,
}

impl EventService {
    pub fn new(repo: EventRepository) -> Self {
        Self { repo }
    }

    pub async fn track_events(&self, events: Vec<IncomingEvent>) -> Result<usize, AppError> {
        if events.is_empty() {
            return Ok(0);
        }

        let rows: Vec<ClickEventRow> = events.into_iter().map(ClickEventRow::from).collect();
        let inserted = rows.len();

        self.repo.insert_events(rows).await?;

        Ok(inserted)
    }

    pub async fn heatmap(
        &self,
        page_url: String,
        event_type: String,
    ) -> Result<Vec<HeatmapPoint>, AppError> {
        self.repo.get_heatmap(&page_url, &event_type).await
    }

    pub async fn navigation_flow(&self, user_id: String) -> Result<Vec<NavigationStep>, AppError> {
        self.repo.get_navigation_flow(&user_id).await
    }

    pub async fn navigation_stats(&self) -> Result<Vec<NavigationStats>, AppError> {
        self.repo.get_navigation_stats().await
    }

    pub async fn user_journey(&self, user_id: String) -> Result<Vec<NavigationStep>, AppError> {
        self.repo.get_user_journey(&user_id).await
    }

    pub async fn click_logs(&self) -> Result<Vec<ClickLog>, AppError> {
        self.repo.get_click_logs().await
    }

    pub async fn click_count(&self, user_id: String) -> Result<Vec<ClickCountResult>, AppError> {
        self.repo.get_click_count(&user_id).await
    }

    pub async fn funnel(&self, steps: Vec<String>) -> Result<Vec<FunnelStepResult>, AppError> {
        let counts = self.repo.get_funnel_counts(&steps).await?;

        Ok(counts
            .into_iter()
            .map(|(step_name, sessions_reached)| FunnelStepResult {
                step_name,
                sessions_reached,
            })
            .collect())
    }

    pub async fn scroll_depth(&self, page_url: String) -> Result<Vec<ScrollDepthResult>, AppError> {
        self.repo.get_scroll_depth(&page_url).await
    }

    pub async fn countries(&self) -> Result<Vec<LocationStats>, AppError> {
        self.repo.get_countries().await
    }

    pub async fn regions(&self, country: String) -> Result<Vec<LocationStats>, AppError> {
        self.repo.get_regions(&country).await
    }

    pub async fn cities(&self, region: String) -> Result<Vec<LocationStats>, AppError> {
        self.repo.get_cities(&region).await
    }

    pub async fn user_report(&self, user_id: String) -> Result<UserProfileReport, AppError> {
        let meta = self.repo.get_user_metadata(&user_id).await?.unwrap_or(UserMetadataRow {
            country: "Unknown".to_string(),
            region: "Unknown".to_string(),
            city: "Unknown".to_string(),
            browser: "Unknown".to_string(),
            os: "Unknown".to_string(),
            device_type: "Unknown".to_string(),
        });

        let navigation_flow = self.repo.get_navigation_flow_detailed(&user_id).await?;
        let clicks = self.repo.get_user_clicks_detailed(&user_id).await?;
        let exit_way = self.repo.get_user_exit_way(&user_id).await?;

        Ok(UserProfileReport {
            user_id,
            country: meta.country,
            region: meta.region,
            city: meta.city,
            browser: meta.browser,
            os: meta.os,
            device_type: meta.device_type,
            navigation_flow,
            clicks,
            exit_way,
        })
    }
}