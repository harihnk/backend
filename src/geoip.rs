use serde::Deserialize;
use crate::errors::AppError;

#[derive(Debug, Clone, Default)]
pub struct GeoLocation {
    pub ip: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub city: String,
}

#[derive(Clone)]
pub struct GeoLocationService {
    client: reqwest::Client,
}

impl GeoLocationService {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("failed to build HTTP client for geo lookup");

        Self { client }
    }

    pub async fn lookup(&self, ip: &str) -> Result<GeoLocation, AppError> {
        // Primary: ipinfo.io (better accuracy for Indian IPs, HTTPS)
        match self.lookup_ipinfo(ip).await {
            Ok(geo) => return Ok(geo),
            Err(e) => {
                tracing::warn!("ipinfo.io lookup failed for {}: {}, trying ip-api.com", ip, e);
            }
        }

        // Fallback: ip-api.com
        self.lookup_ipapi(ip).await
    }

    async fn lookup_ipinfo(&self, ip: &str) -> Result<GeoLocation, AppError> {
        let url = format!("https://ipinfo.io/{}/json", ip);

        let response: IpInfoResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("ipinfo request failed: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("ipinfo parse failed: {e}")))?;

        if response.bogon.unwrap_or(false) {
            return Err(AppError::Internal("bogon IP".to_string()));
        }

        Ok(GeoLocation {
            ip: response.ip.unwrap_or_else(|| ip.to_string()),
            country_code: null_to_unknown(response.country.clone()),
            country: null_to_unknown(response.country),
            region: null_to_unknown(response.region),
            city: null_to_unknown(response.city),
        })
    }

    async fn lookup_ipapi(&self, ip: &str) -> Result<GeoLocation, AppError> {
        let url = format!(
            "http://ip-api.com/json/{}?fields=status,message,query,country,countryCode,regionName,city",
            ip
        );

        let response: IpApiResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("ip-api request failed: {e}")))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("ip-api parse failed: {e}")))?;

        if response.status != "success" {
            return Err(AppError::Internal(
                response.message.unwrap_or_else(|| "ip-api lookup failed".to_string()),
            ));
        }

        Ok(GeoLocation {
            ip: response.query.unwrap_or_else(|| ip.to_string()),
            country: null_to_unknown(response.country),
            country_code: null_to_unknown(response.country_code),
            region: null_to_unknown(response.region_name),
            city: null_to_unknown(response.city),
        })
    }
}

#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    ip: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    bogon: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    status: String,
    message: Option<String>,
    query: Option<String>,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    #[serde(rename = "regionName")]
    region_name: Option<String>,
    city: Option<String>,
}

fn null_to_unknown(value: Option<String>) -> String {
    value
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}