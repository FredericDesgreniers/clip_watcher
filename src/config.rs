#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub client_id: String,
    pub bearer: Option<String>,
    pub channel: String
}
