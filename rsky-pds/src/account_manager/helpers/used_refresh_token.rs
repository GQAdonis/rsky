use crate::db::DbConn;
use anyhow::Result;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use time::OffsetDateTime;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::pds::used_refresh_token)]
struct NewUsedRefreshToken {
    jti: String,
    did: String,
    expires_at: String,
    created_at: String,
}

/// Record a JTI as consumed. Returns `Ok(true)` if newly inserted,
/// `Ok(false)` if already present (indicating a replay attempt).
pub async fn insert(jti: &str, did: &str, expires_at: &str, db: &DbConn) -> Result<bool> {
    use crate::schema::pds::used_refresh_token::dsl as UrtSchema;

    let now = OffsetDateTime::now_utc();
    let created_at = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    );

    let row = NewUsedRefreshToken {
        jti: jti.to_string(),
        did: did.to_string(),
        expires_at: expires_at.to_string(),
        created_at,
    };

    let result = db
        .run(move |conn| {
            diesel::insert_into(UrtSchema::used_refresh_token)
                .values(&row)
                .execute(conn)
        })
        .await;

    match result {
        Ok(_) => Ok(true),
        Err(diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Ok(false)
        }
        Err(e) => Err(anyhow::Error::from(e)),
    }
}

/// Returns `true` if the JTI has already been used.
pub async fn exists(jti: &str, db: &DbConn) -> Result<bool> {
    use crate::schema::pds::used_refresh_token::dsl as UrtSchema;

    let jti = jti.to_string();
    let count: i64 = db
        .run(move |conn| {
            UrtSchema::used_refresh_token
                .filter(UrtSchema::jti.eq(&jti))
                .count()
                .get_result(conn)
        })
        .await?;

    Ok(count > 0)
}

/// Deletes rows whose `expires_at` is before `now_str` (ISO-8601 UTC).
/// Safe to call from a background task; failures are non-fatal.
pub async fn prune_expired(now_str: &str, db: &DbConn) -> Result<usize> {
    use crate::schema::pds::used_refresh_token::dsl as UrtSchema;

    let now_str = now_str.to_string();
    let deleted = db
        .run(move |conn| {
            diesel::delete(UrtSchema::used_refresh_token.filter(UrtSchema::expires_at.lt(&now_str)))
                .execute(conn)
        })
        .await?;

    Ok(deleted)
}
