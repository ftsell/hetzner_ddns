use reqwest::{
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
    Url,
};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref API_URL: Url = {
        Url::parse("https://dns.hetzner.com/api/v1/").unwrap()
    };
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    /// The number of the last available page
    pub last_page: usize,
    /// The number of the current page
    pub page: usize,
    /// The number of entries returned per page
    pub per_page: usize,
    /// The total number of entries defined over all pages
    pub total_entries: usize,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMeta {
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub meta: ResponseMeta,
    #[serde(flatten)]
    pub content: T,
}

#[derive(Debug, Deserialize)]
pub struct ZoneResponse {
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub project: String,
    pub paused: bool,
    pub permission: String,
    pub is_secondary_dns: bool,
    pub ns: Vec<String>,
    pub records_count: u64,
    pub registrar: String,
    pub status: String,
    pub ttl: u64,
    pub created: String,
    pub modified: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordResponse {
    pub records: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Record {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
    pub ttl: Option<u64>,
    pub zone_id: String,
    pub created: String,
    pub modified: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateRecordData {
    pub name: String,
    pub ttl: u64,
    #[serde(rename = "type")]
    pub typ: String,
    pub value: String,
    pub zone_id: String,
}

pub struct Client {
    req_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: &str) -> Self {
        let req_client = reqwest::Client::builder()
            .default_headers({
                let mut map = HeaderMap::new();

                let mut auth_value = HeaderValue::from_str(api_key)
                    .expect("Api-Key is not a valid HTTP header value");
                auth_value.set_sensitive(true);
                map.insert("Auth-API-Token", auth_value);

                map
            })
            .redirect(Policy::none())
            .build()
            .expect("Could not build reqwest client");

        Self { req_client }
    }

    /// .Returns paginated zones associated with the user.
    ///
    /// Limited to 100 zones per request.
    ///
    /// # Parameters
    /// - `name`: Full name of a zone. Will return an array with the results or return an error.
    /// - `search_name`: Partial name of a zone.
    /// - `page`: A page parameter specifies the page to fetch.
    ///    The number of the first page and default is 1.
    pub async fn get_all_zones(
        &mut self,
        name: Option<&str>,
        search_name: Option<&str>,
        page: Option<usize>,
    ) -> eyre::Result<Response<ZoneResponse>> {
        Ok(self
            .req_client
            .get(API_URL.join("zones").unwrap())
            .query(&[("name", name)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Returns all records associated with given zone
    pub async fn get_all_records(&mut self, zone_id: &str) -> eyre::Result<RecordResponse> {
        Ok(self
            .req_client
            .get(API_URL.join("records").unwrap())
            .query(&[("zone_id", zone_id)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Update all data of a DNS record
    pub async fn update_record(
        &mut self,
        record_id: &str,
        data: &UpdateRecordData,
    ) -> eyre::Result<()> {
        self.req_client
            .put(API_URL.join("records/").unwrap().join(record_id).unwrap())
            .json(data)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
