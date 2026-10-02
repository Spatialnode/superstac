use std::process::Command;
use superstac_core::models::settings::Settings;

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config = serde_json::json!({
        "catalogs": [{"id": "test", "url": "http://127.0.0.1:9"}],
        "providers": [], "settings": Settings::default()
    });
    std::fs::write(dir.path().join("superstac.yml"), config.to_string()).unwrap();
    dir
}

#[cfg(not(feature = "geoparquet"))]
#[test]
fn unavailable_feature_returns_an_actionable_error() {
    let dir = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_superstac"))
        .current_dir(dir.path())
        .args(["--geoparquet", "test=items.parquet", "search"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--features geoparquet"));
}

#[cfg(feature = "geoparquet")]
#[test]
fn snapshot_cli_returns_items_and_reports_corrupt_files_as_failures() {
    let dir = fixture();
    let path = dir.path().join("items.parquet");
    let mut item = stac::Item::new("scene");
    item.collection = Some("test-collection".into());
    item.geometry = Some(
        serde_json::from_value(serde_json::json!({
            "type": "Point", "coordinates": [1.0, 1.0]
        }))
        .unwrap(),
    );
    stac::geoparquet::into_writer(std::fs::File::create(&path).unwrap(), vec![item]).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_superstac"))
            .current_dir(dir.path())
            .args([
                "--json",
                "--geoparquet",
                "test=items.parquet",
                "search",
                "--collection",
                "test-collection",
            ])
            .output()
            .unwrap()
    };
    let output = run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["items"][0]["item"]["id"], "scene");
    assert_eq!(value["metadata"]["catalogs_succeeded"], 1);
    std::fs::write(path, b"broken").unwrap();
    let output = run();
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["metadata"]["catalogs_failed"], 1);
}
