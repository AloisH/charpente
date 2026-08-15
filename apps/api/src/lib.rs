//! Library crate so integration tests (and the thin `main.rs`) build the exact
//! production router.

pub mod app;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod mailer;
pub mod openapi;
pub mod pagination;
pub mod routes;
pub mod seed;
pub mod state;
pub mod static_files;
pub mod storage;
pub mod telemetry;
