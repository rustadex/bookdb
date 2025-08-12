use std::{env, fs, path::PathBuf};


pub fn build_const_from_dir(src_dir:mut &str, out_path:&str) -> (){

  let mut buf = String::new();

  for entry in fs::read_dir(&src_dir).unwrap() {
    let path = entry.unwrap().path();
    if path.extension().and_then(|s| s.to_str()) == Some("sql") {

      let file_name = path.file_name().unwrap().to_str().unwrap();     
      // e.g., resolve_project_id.sql
      let stem = path.file_stem().unwrap().to_str().unwrap();         
        // e.g., resolve_project_id

      // CONST NAME: uppercase + non-alnum -> _
      let const_name: String = stem.chars()
          .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
          .collect();

      // emit: pub const RESOLVE_PROJECT_ID: &str = include_str!("src/db/data/sqlv2/resolve_project_id.sql");
      buf.push_str(&format!(
          "pub const {}: &str = include_str!(\"src/db/data/{}/{}\");\n",
          const_name, sql_version, file_name
      ));
    }
  }

  fs::write(&out_path, buf).unwrap();

}



fn main() {
    // ----- read Cargo.toml
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();

    // ----- parse version from [package.metadata.sql]
    let doc: toml::Value = toml::from_str(&cargo_toml).unwrap();
    let sql_version = doc
        .get("package").and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("sql")).and_then(|s| s.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("sqlv2"); // default if missing

    // make the version available in code
    println!("cargo:rustc-env=SQL_VERSION={}", sql_version);

    // ----- scan folder and generate consts
    let sql_dir = manifest_dir.join("src/db/data").join(sql_version);
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sql_consts.rs");


    build_const_from_dir(&sql_dir);


    // ----- rebuild rules
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/service/db/data/{}", sql_version);
    println!("cargo:rerun-if-changed=Cargo.toml");
    

  }
