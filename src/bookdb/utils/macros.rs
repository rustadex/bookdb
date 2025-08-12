

// ---------------------------------------------------------
// depends on build.rs
// generate sql constants based on FILE_NAME.sql
#[macro_export]
macro_rules! include_sql_mod {
    () => {
        pub mod sql {
            pub const SQL_VERSION: &str = env!("SQL_VERSION");
            include!(concat!(env!("OUT_DIR"), "/sql_consts.rs"));
        }
    };
}
// ---------------------------------------------------------
