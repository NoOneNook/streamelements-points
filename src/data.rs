use reqwest;
use toml;
use csv;

#[derive(Debug, Deserialize, Serialize)]
pub struct Alltime {
    pub _total: u64,
    users: Vec<User>,
}

impl Alltime {
    pub fn users(&self) -> &Vec<User> {
        &self.users
    }

    pub fn into_users(self) -> Vec<User> {
        self.users
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    username: String,
    pub points: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "streamelements_id")] channel: String,
    cutoff: Option<u64>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ActualConfig {
    info: Config,
}

impl ActualConfig {
    pub fn channel(&self) -> &str {
        &self.info.channel
    }

    pub fn cutoff(&self) -> Option<u64> {
        self.info.cutoff
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// There was an error sending a request to the site,
    /// possibly check your internet connection
    Reqwest(reqwest::Error),
    /// There was an error with the I/O of your system
    Io(::std::io::Error),
    /// Unable to read config
    TomlDeserialized(toml::de::Error),
    /// Error with CSV info
    Csv(csv::Error),
}
