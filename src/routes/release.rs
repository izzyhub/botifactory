use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::header,
    http::header::{HeaderMap, HeaderValue, ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::io;
use tokio_util::io::ReaderStream;

use crate::configuration::Settings;
use crate::routes::error::{APIError, Result};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use botifactory_common::{ReleaseBody, ReleaseResponse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct ReleaseRow {
    pub id: i64,
    pub version: String,
    pub hash: Vec<u8>,
    pub path: PathBuf,
    pub channel_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<ReleaseRow> for ReleaseResponse {
    fn from(row: ReleaseRow) -> Self {
        ReleaseResponse {
            id: row.id,
            version: row.version,
            hash: row.hash,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/*
#[derive(Serialize, Deserialize)]
struct CreateRelease {
    version: String,
    contents: Vec<u8>,
}
*/

#[derive(TryFromMultipart)]
pub struct CreateRelease {
    version: String,
    #[form_data(limit = "15MiB")]
    binary: FieldData<NamedTempFile>,
    //binary: Bytes,
}

const fn body_limit() -> usize {
    let base: usize = 10;
    //15 * base.checked_pow(7).expect("Body limit exponent overflowed")
    let res = base.checked_pow(7);
    let result = match res {
        Some(result) => result,
        None => panic!("body limit exponent overflowed"),
    };
    15 * result
}

pub fn router() -> Router<(SqlitePool, Arc<Settings>)> {
    Router::new()
        .route(
            "/:project_name/:channel_name/current",
            get(show_latest_project_release),
        )
        .route(
            "/:project_name/:channel_name/latest",
            get(show_latest_project_release),
        )
        .route(
            "/:project_name/:channel_name/previous",
            get(show_previous_project_release),
        )
        .route("/releases/:id", get(show_project_release))
        .route(
            "/:project_name/:channel_name/new",
            post(create_project_release),
        )
        .layer(DefaultBodyLimit::max(body_limit()))
}

pub async fn show_latest_project_release(
    headers: HeaderMap,
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Response> {
    let release = sqlx::query_as!(
        ReleaseRow,
        r#"
          select releases.id as id,
            releases.version as version,
            releases.hash as hash,
            releases.path as path,
            releases.channel_id as channel_id,
            releases.created_at as created_at,
            releases.updated_at as updated_at
          from releases
            left join release_channel ON releases.channel_id = release_channel.id
            left join projects ON release_channel.project_id = projects.id
          where projects.name = $1
            and release_channel.name = $2
          order by created_at desc
          limit 2
        "#,
        project_name,
        channel_name
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    match headers.get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"*/*") => Ok(Json(ReleaseBody {
            release: release.into(),
        })
        .into_response()),
        Some(b"application/json") => Ok(Json(ReleaseBody {
            release: release.into(),
        })
        .into_response()),
        Some(b"application/octet-stream") => {
            let file = tokio::fs::File::open(release.path.clone())
                .await
                .map_err(|_| APIError::NotFound)?;
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str("application/octet-stream")?,
            );
            let filename = release.path.file_name().ok_or(APIError::InternalError)?;
            let filename = filename.to_str().ok_or(APIError::InternalError)?;

            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))?,
            );
            Ok((headers, body).into_response())
        }
        None => Err(APIError::RequestError),
        Some(_) => Err(APIError::UnsupportedMediaType),
    }
}

pub async fn show_previous_project_release(
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Json<ReleaseBody>> {
    let releases = sqlx::query_as!(
        ReleaseRow,
        r#"
          select releases.id as id,
          releases.version as version,
          releases.hash as hash,
          releases.path as path,
          releases.channel_id as channel_id,
          releases.created_at as created_at,
          releases.updated_at as updated_at
          from releases
          left join release_channel ON releases.channel_id = release_channel.id
          left join projects ON release_channel.project_id = projects.id
          where projects.name = $1
          and release_channel.name = $2
          order by created_at desc
          limit 2
        "#,
        project_name,
        channel_name
    )
    .fetch_all(&db)
    .await?;

    let previous_release = releases.iter().last().ok_or(APIError::NotFound)?;
    Ok(Json(ReleaseBody {
        release: previous_release.clone().into(),
    }))
}

pub async fn show_project_release(
    Path(id): Path<i64>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Json<ReleaseBody>> {
    let release = sqlx::query_as!(
        ReleaseRow,
        r#"
          select id,
          version,
          hash,
          path,
          channel_id,
          created_at,
          updated_at
          from releases
          where id = $1  
        "#,
        id,
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    Ok(Json(ReleaseBody {
        release: release.into(),
    }))
}

pub async fn create_project_release(
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    TypedMultipart(CreateRelease { version, binary }): TypedMultipart<CreateRelease>,
) -> Result<Json<ReleaseBody>> {
    let channel_path: PathBuf = [
        &settings.application.release_path,
        &PathBuf::from(&project_name),
        &PathBuf::from(&channel_name),
    ]
    .iter()
    .collect();
    create_dir_all(channel_path.clone())?;
    let release_path: PathBuf = [
        channel_path,
        PathBuf::from(format!("{project_name}v{version}")),
    ]
    .iter()
    .collect();
    let mut persisted_file = binary.contents.persist(release_path.clone())?;

    let mut hasher = Sha256::new();
    let _ = io::copy(&mut persisted_file, &mut hasher)?;
    let hash = hasher.finalize();
    let hash = format!("{:x}", hash);

    let release_path = release_path.to_str().ok_or(APIError::InternalError)?;

    let response = sqlx::query!(
        r#"
          INSERT INTO releases
          (version, hash, path, channel_id) VALUES ($1, $2, $3, 
            (SELECT id FROM release_channel WHERE name = $4 and project_id =
              (SELECT id from projects where name=$5))
          )
          RETURNING id, version, hash, created_at, updated_at
        "#,
        version,
        hash,
        release_path,
        channel_name,
        project_name
    )
    .fetch_one(&db)
    .await?;

    let release = ReleaseResponse {
        id: response.id,
        version: response.version,
        hash: response.hash,
        created_at: response.created_at,
        updated_at: response.updated_at,
    };

    Ok(Json(ReleaseBody { release }))
}
