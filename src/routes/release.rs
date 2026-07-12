use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, State},
    http::header,
    http::header::{HeaderMap, HeaderValue, ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::io::{Read, Seek, SeekFrom};
use tokio_util::io::ReaderStream;

use crate::auth::RequireApiToken;
use crate::configuration::Settings;
use crate::release_ordering::{latest_release, previous_release, ReleaseCandidate};
use crate::routes::error::{conflict_on_unique, APIError, Result};
use crate::validation::{parse_version, ResourceName};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use botifactory_types::{ReleaseBody, ReleaseResponse};
use semver::Version;
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
    pub hash: String,
    pub path: PathBuf,
    pub channel_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<ReleaseRow> for ReleaseResponse {
    type Error = APIError;

    // A malformed version in the database must not panic the handler; it
    // becomes a logged 500 instead.
    fn try_from(row: ReleaseRow) -> Result<Self> {
        Ok(ReleaseResponse {
            id: row.id,
            version: Version::parse(&row.version)?,
            hash: row.hash,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

/// Pairs the rows with parsed versions for `release_ordering`; rows whose
/// stored version does not parse are logged and skipped rather than failing
/// the whole request.
fn release_candidates(rows: &[ReleaseRow]) -> Vec<ReleaseCandidate> {
    rows.iter()
        .filter_map(|row| match Version::parse(&row.version) {
            Ok(version) => Some(ReleaseCandidate {
                id: row.id,
                version,
                created_at: row.created_at,
            }),
            Err(error) => {
                tracing::warn!(
                    "release {} has an unparseable version {:?}: {error}",
                    row.id,
                    row.version
                );
                None
            }
        })
        .collect()
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
            "/{project_name}/{channel_name}/current",
            get(show_latest_project_release),
        )
        .route(
            "/{project_name}/{channel_name}/latest",
            get(show_latest_project_release),
        )
        .route(
            "/{project_name}/{channel_name}/previous",
            get(show_previous_project_release),
        )
        .route("/releases/{id}", get(show_project_release))
        .route(
            "/{project_name}/{channel_name}/new",
            post(create_project_release),
        )
        .layer(DefaultBodyLimit::max(body_limit()))
}

pub async fn show_latest_project_release(
    headers: HeaderMap,
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, _settings)): State<(SqlitePool, Arc<Settings>)>,
) -> Result<Response> {
    let rows = sqlx::query_as!(
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
        "#,
        project_name,
        channel_name
    )
    .fetch_all(&db)
    .await?;

    // Latest is the highest semver, not the newest upload: uploading an old
    // version must not make devices "upgrade" backwards.
    let candidates = release_candidates(&rows);
    let latest = latest_release(&candidates).ok_or(APIError::NotFound)?;
    let release = rows
        .iter()
        .find(|row| row.id == latest.id)
        .cloned()
        .ok_or(APIError::InternalError)?;

    match headers.get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"*/*") => Ok(Json(ReleaseBody {
            release: release.try_into()?,
        })
        .into_response()),
        Some(b"application/json") => Ok(Json(ReleaseBody {
            release: release.try_into()?,
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
        None => {
            tracing::error!("no response type");
            Err(APIError::RequestError)
        }
        Some(_bad_type) => {
            tracing::error!("unsupported header type");
            //tracing::error!("unsupported header type: {bad_type}");
            Err(APIError::UnsupportedMediaType)
        }
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
        "#,
        project_name,
        channel_name
    )
    .fetch_all(&db)
    .await?;

    // Previous is the newest release with a version strictly lower than the
    // latest — the release a device would roll back to. With a single release
    // (or every release at the same version) there is nothing to roll back
    // to, so this is a 404.
    let candidates = release_candidates(&releases);
    let previous = previous_release(&candidates).ok_or(APIError::NotFound)?;
    let release = releases
        .iter()
        .find(|row| row.id == previous.id)
        .cloned()
        .ok_or(APIError::InternalError)?;
    Ok(Json(ReleaseBody {
        release: release.try_into()?,
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
        release: release.try_into()?,
    }))
}

pub async fn create_project_release(
    _auth: RequireApiToken,
    Path((project_name, channel_name)): Path<(String, String)>,
    State((db, settings)): State<(SqlitePool, Arc<Settings>)>,
    TypedMultipart(CreateRelease { version, binary }): TypedMultipart<CreateRelease>,
) -> Result<Json<ReleaseBody>> {
    // All three values end up as path components under release_path.
    // ResourceName rejects separators/dots, and rendered semver cannot
    // contain them, so the joined path cannot escape the release directory.
    let project_name = ResourceName::parse(&project_name)?;
    let channel_name = ResourceName::parse(&channel_name)?;
    let version = parse_version(&version)?.to_string();

    // Resolve the channel explicitly so bad names are a 404 rather than a
    // NOT NULL constraint failure (500) after the file already landed.
    let channel_name_str = channel_name.as_str();
    let project_name_str = project_name.as_str();
    let channel_id = sqlx::query_scalar!(
        r#"
          select release_channel.id from release_channel
          join projects on release_channel.project_id = projects.id
          where release_channel.name = $1 and projects.name = $2
        "#,
        channel_name_str,
        project_name_str
    )
    .fetch_optional(&db)
    .await?
    .ok_or(APIError::NotFound)?;

    // A version maps to exactly one file path, so a re-upload of an existing
    // version would silently overwrite the file older rows point at. Refuse.
    let existing = sqlx::query_scalar!(
        r#"select count(*) as "count: i64" from releases where channel_id = $1 and version = $2"#,
        channel_id,
        version
    )
    .fetch_one(&db)
    .await?;
    if existing > 0 {
        return Err(APIError::Conflict);
    }

    let channel_dir = settings
        .application
        .release_path
        .join(project_name.as_str())
        .join(channel_name.as_str());
    let release_file = channel_dir.join(format!("{}v{}", project_name.as_str(), version));
    // Defense in depth: the validated components above make escape
    // impossible, but never write outside the release directory.
    if !release_file.starts_with(&settings.application.release_path) {
        return Err(APIError::RequestError);
    }

    // Hash the upload before anything is persisted, so the stored hash always
    // describes the file that lands on disk.
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    let mut upload = binary.contents.as_file();
    upload.seek(SeekFrom::Start(0))?;
    loop {
        let read = upload.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let hash = hex::encode(hasher.finalize());

    let release_path = release_file.to_str().ok_or(APIError::InternalError)?;

    create_dir_all(&channel_dir)?;
    let response = sqlx::query!(
        r#"
          INSERT INTO releases
          (version, hash, path, channel_id) VALUES ($1, $2, $3, $4)
          RETURNING id, version, hash, created_at, updated_at
        "#,
        version,
        hash,
        release_path,
        channel_id
    )
    .fetch_one(&db)
    .await
    .map_err(conflict_on_unique)?;
    let id = response.id.ok_or(APIError::InternalError)?;

    if let Err(persist_error) = binary.contents.persist(&release_file) {
        // Compensate: the row exists but the file never landed.
        if let Err(delete_error) = sqlx::query!(r#"delete from releases where id = $1"#, id)
            .execute(&db)
            .await
        {
            tracing::error!(
                "failed to delete release row {id} after a persist failure: {delete_error}"
            );
        }
        return Err(APIError::PersistBinaryError(persist_error));
    }

    let release = ReleaseResponse {
        id,
        version: Version::parse(&response.version)?,
        hash: response.hash,
        created_at: response.created_at,
        updated_at: response.updated_at,
    };

    Ok(Json(ReleaseBody { release }))
}
