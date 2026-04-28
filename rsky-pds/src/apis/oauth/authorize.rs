use rocket::http::Status;
use rocket::response::Redirect;

/// Authorization endpoint — returns a 501 until full OAuth is implemented.
/// AT Protocol requires PAR, so clients must use /oauth/par first.
#[rocket::get("/oauth/authorize?<_query..>")]
pub fn oauth_authorize(_query: Option<&str>) -> (Status, &'static str) {
    (
        Status::NotImplemented,
        "OAuth authorization endpoint is not yet implemented. Use PAR first.",
    )
}
