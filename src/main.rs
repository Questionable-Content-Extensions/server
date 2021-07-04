//! Questionable Content Extensions server.

// <editor-fold desc="Coding conventions" defaultstate="collapsed">
// Coding conventions
//
// Deny (don't do this)
#![deny(anonymous_parameters)]
#![deny(elided_lifetimes_in_paths)]
#![deny(ellipsis_inclusive_range_patterns)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
//#![deny(unused)]
//
// Warn (try not to do this)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(variant_size_differences)]
//
// Clippy conventions
//
// Deny (don't do this)
#![deny(clippy::cast_lossless)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::empty_enum)]
#![deny(clippy::enum_glob_use)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::manual_filter_map)]
#![deny(clippy::filter_map_next)]
#![deny(clippy::if_not_else)]
#![deny(clippy::invalid_upcast_comparisons)]
#![deny(clippy::items_after_statements)]
#![deny(clippy::large_digit_groups)]
#![deny(clippy::map_flatten)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::mut_mut)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::single_match_else)]
#![deny(clippy::string_add_assign)]
#![deny(clippy::type_repetition_in_bounds)]
#![deny(clippy::unseparated_literal_suffix)]
#![deny(clippy::unused_self)]
#![deny(clippy::use_self)] // Sometimes gives false positives; feel free to disable.
#![deny(clippy::used_underscore_binding)]
//
// Warn (try not to do this)
#![warn(clippy::must_use_candidate)]
#![warn(clippy::pub_enum_variant_names)]
#![warn(clippy::shadow_unrelated)]
#![warn(clippy::similar_names)]
#![warn(clippy::too_many_lines)]
// </editor-fold>

use actix_web::{web, App, HttpServer};
use anyhow::{Context as _, Result};
use util::Environment;

mod controllers;
mod database;
pub(crate) mod models;

mod util;

#[actix_web::main]
async fn main() -> Result<()> {
    Environment::init();

    // Initialize logging
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "actix_web=info,qcext_server=info");
    }
    pretty_env_logger::init();

    let bind = format!("localhost:{}", Environment::port());
    println!("Starting server at: {}", &bind);

    let db_pool = database::create_db_pool().await;

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(actix_web::middleware::Logger::default())
            .service(web::scope("/api").configure(controllers::api::configure))
        //.wrap(crate::middleware::authentication::Authentication)
        //.service(web::scope("/auth").configure(controllers::auth::configure))
        //.service(web::scope("/location").configure(controllers::location::configure))
        //.service(web::scope("/graphql").configure(controllers::graphql::configure))
    })
    .bind(&bind)?
    .run()
    .await
    .context("actix_web::HttpServer crashed")?;

    Ok(())
}
