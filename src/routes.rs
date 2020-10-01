use crate::{config::Config, db::User, reject::TryExt};
use bytes::Bytes;
use sled::Db;
use warp::{http::Uri, Filter, Rejection, Reply};

pub fn handler(
    config: &'static Config,
    db: &'static Db,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Copy + Send + Sync + 'static {
    let filite = warp::path!(String)
        .and(warp::get())
        .and_then(move |id| filite(id, db));

    let post_file = warp::path!("f")
        .and(warp::post())
        .and(crate::auth::required(db, config))
        .and(warp::body::bytes())
        .and(warp::header("Content-Type"))
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, data, mime, len| post_file(user, data, mime, len, db));
    let put_file = warp::path!("f" / String)
        .and(warp::put())
        .and(crate::auth::required(db, config))
        .and(warp::body::bytes())
        .and(warp::header("Content-Type"))
        .and_then(move |id, user, data, mime| put_file(id, user, data, mime, db));

    let post_link = warp::path!("l")
        .and(warp::post())
        .and(crate::auth::required(db, config))
        .and(crate::util::body())
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, location, len| post_link(user, location, len, db));
    let put_link = warp::path!("l" / String)
        .and(warp::put())
        .and(crate::auth::required(db, config))
        .and(crate::util::body())
        .and_then(move |id, user, location| put_link(id, user, location, db));

    let post_text = warp::path!("t")
        .and(warp::post())
        .and(crate::auth::required(db, config))
        .and(crate::util::body())
        .and(warp::header::optional("X-ID-Length"))
        .and_then(move |user, data, len| post_text(user, data, len, db));
    let put_text = warp::path!("t" / String)
        .and(warp::put())
        .and(crate::auth::required(db, config))
        .and(crate::util::body())
        .and_then(move |id, user, data| put_text(id, user, data, db));

    filite
        .or(post_file)
        .or(put_file)
        .or(post_link)
        .or(put_link)
        .or(post_text)
        .or(put_text)
}

async fn filite(id: String, db: &Db) -> Result<impl Reply, Rejection> {}

async fn post_file(
    user: User,
    data: Bytes,
    mime: String,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = crate::db::random_id(len.unwrap_or(8), db).or_500()?;
    put_file(id, user, data, mime, db).await
}

async fn put_file(
    id: String,
    user: User,
    data: Bytes,
    mime: String,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    crate::db::insert_file(&id, user.id, data.to_vec(), mime, db)
        .or_500()?
        .or_409()?;
    Ok(id)
}

async fn post_link(
    user: User,
    location: Uri,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = crate::db::random_id(len.unwrap_or(8), db).or_500()?;
    put_link(id, user, location, db).await
}

async fn put_link(id: String, user: User, location: Uri, db: &Db) -> Result<impl Reply, Rejection> {
    crate::db::insert_link(&id, user.id, location.to_string(), db)
        .or_500()?
        .or_409()?;
    Ok(id)
}

async fn post_text(
    user: User,
    data: String,
    len: Option<usize>,
    db: &Db,
) -> Result<impl Reply, Rejection> {
    let id = crate::db::random_id(len.unwrap_or(8), db).or_500()?;
    put_text(id, user, data, db).await
}

async fn put_text(id: String, user: User, data: String, db: &Db) -> Result<impl Reply, Rejection> {
    crate::db::insert_text(&id, user.id, data, db)
        .or_500()?
        .or_409()?;
    Ok(id)
}
