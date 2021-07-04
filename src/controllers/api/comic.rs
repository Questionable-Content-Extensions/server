use actix_web::{error, web, HttpResponse, Result};
use futures::TryStreamExt;

use crate::database::DbPool;
use crate::models::Comic;

pub(crate) fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(all)));
}

async fn all(conn: web::Data<DbPool>) -> Result<HttpResponse> {
    let comics: Vec<Comic> = sqlx::query_as::<_, Comic>("SELECT * FROM `comic` ORDER BY id ASC")
        .fetch(&**conn)
        .try_collect()
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(comics))
}
