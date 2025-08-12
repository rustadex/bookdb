
include_sql_mod!(); // expands to: pub mod sql { ... } at this path

// access as: crate::bookdb::service::db::sql::SQL_VERSION
// and crate::bookdb::service::db::sql::RESOLVE_PROJECT_ID, etc.
