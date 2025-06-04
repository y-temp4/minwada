use serde::Deserialize;

#[derive(Deserialize)]
pub struct ThreadQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
