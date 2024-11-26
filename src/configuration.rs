use config::Config;
use serde::{Deserialize, Deserializer};
//use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::ConnectOptions;

use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    pub log_level: LevelFilter,
    pub release_path: PathBuf,
}

impl Default for ApplicationSettings {
    fn default() -> Self {
        ApplicationSettings {
            port: 3000,
            host: "localhost".into(),
            log_level: LevelFilter::TRACE,
            release_path: PathBuf::from("/releases"),
        }
    }
}

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        LevelFilter::from_str(string.to_lowercase().as_str())
            .map_err(|err| Error::custom(err.to_string()))
    })
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq)]
pub struct DatabaseSettings {
    pub url: String,
    #[serde(deserialize_with = "deserialize_log_level")]
    pub log_level: LevelFilter,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        DatabaseSettings {
            url: "file://botifactory.db".to_string(),
            log_level: LevelFilter::INFO,
        }
    }
}

impl DatabaseSettings {
    pub fn without_db(&self) -> SqliteConnectOptions {
        println!("w/o db database_url: {}", self.url.as_str());
        SqliteConnectOptions::from_str(self.url.as_str())
            .expect("Failed to parse databse url")
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true)
            .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Full)
            .log_statements(convert_to_log_level_filter(self.log_level))
    }

    pub fn with_db(&self) -> SqliteConnectOptions {
        println!("w/ db database_url: {}", self.url.as_str());
        self.without_db().filename(&self.url)
    }
}

pub fn convert_to_log_level_filter(level: LevelFilter) -> log::LevelFilter {
    match level {
        LevelFilter::OFF => log::LevelFilter::Off,
        LevelFilter::ERROR => log::LevelFilter::Error,
        LevelFilter::WARN => log::LevelFilter::Warn,
        LevelFilter::INFO => log::LevelFilter::Info,
        LevelFilter::DEBUG => log::LevelFilter::Debug,
        LevelFilter::TRACE => log::LevelFilter::Trace,
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Unable to get the current directory");
    let configuration_directory = base_path.join("configuration");

    let app_environment_key = "BOTIFACTORY_APP_ENVIRONMENT";
    let environment: Environment = std::env::var(app_environment_key)
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse BOTIFACTORY_APP_ENVIRONMENT");

    Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")))
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        .add_source(
            config::Environment::with_prefix("BOTIFACTORY_APP")
                //.try_parsing(true)
                .convert_case(config::Case::Snake)
                .separator("__")
                .list_separator(" "),
        )
        .build()?
        .try_deserialize()
}

pub enum Environment {
    Development,
    Test,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Test => "test",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use 'development', 'test', or 'production'."
            )),
        }
    }
}
