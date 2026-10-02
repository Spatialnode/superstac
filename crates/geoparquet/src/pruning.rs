//! Statistics are only used when exclusion is certain; absent/null stats scan normally.
use parquet::{
    basic::{LogicalType, TimeUnit},
    file::{metadata::RowGroupMetaData, statistics::Statistics},
};
use stac::api::Search;

pub(crate) fn may_match(group: &RowGroupMetaData, search: &Search) -> bool {
    for column in group.columns() {
        let field = column.column_path().string();
        let Some(stats) = column.statistics() else {
            continue;
        };
        if field == "id" || field == "collection" {
            let values = if field == "id" {
                &search.ids
            } else {
                &search.collections
            };
            if values.is_empty() {
                continue;
            }
            if let Statistics::ByteArray(stats) = stats {
                if let (Some(min), Some(max)) = (stats.min_opt(), stats.max_opt()) {
                    if !values
                        .iter()
                        .any(|v| v.as_bytes() >= min.data() && v.as_bytes() <= max.data())
                    {
                        return false;
                    }
                }
            }
        }
        // A nullable datetime may represent an interval item. Never exclude those
        // using the non-null rows' instant statistics.
        if field == "datetime" && stats.null_count_opt() == Some(0) {
            if let (
                Some(datetime),
                Statistics::Int64(stats),
                Some(LogicalType::Timestamp { unit, .. }),
            ) = (
                &search.items.datetime,
                stats,
                column.column_descr().logical_type(),
            ) {
                if let (Ok((start, end)), Some(min), Some(max)) = (
                    stac::datetime::parse(datetime),
                    stats.min_opt(),
                    stats.max_opt(),
                ) {
                    let scale = match unit {
                        TimeUnit::MILLIS(_) => 1_000_000i128,
                        TimeUnit::MICROS(_) => 1_000,
                        TimeUnit::NANOS(_) => 1,
                    };
                    let ns = |t: chrono::DateTime<chrono::FixedOffset>| {
                        i128::from(t.timestamp()) * 1_000_000_000
                            + i128::from(t.timestamp_subsec_nanos())
                    };
                    if start.is_some_and(|t| ns(t) > i128::from(*max) * scale)
                        || end.is_some_and(|t| ns(t) < i128::from(*min) * scale)
                    {
                        return false;
                    }
                }
            }
        }
        // GeoParquet covering bbox struct. Null or missing bounds are not pruned.
        if stats.null_count_opt() == Some(0) {
            if let (Some(bbox), Statistics::Double(stats)) = (search.items.bbox, stats) {
                if bbox.is_valid() {
                    if let (Some(min), Some(max)) = (stats.min_opt(), stats.max_opt()) {
                        if min.is_finite()
                            && max.is_finite()
                            && match field.as_str() {
                                "bbox.xmin" => *min > bbox.xmax(),
                                "bbox.xmax" => *max < bbox.xmin(),
                                "bbox.ymin" => *min > bbox.ymax(),
                                "bbox.ymax" => *max < bbox.ymin(),
                                _ => false,
                            }
                        {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{visit_items, GeoParquetBackend};
    use std::{collections::HashMap, fs::File, sync::atomic::AtomicBool, time::Instant};
    use superstac_core::models::catalog::Catalog;
    use superstac_search::{
        backend::{BackendSearchOptions, SearchBackend},
        query::SearchQuery,
        translator::to_stac_search,
    };

    fn fixture(path: &std::path::Path, count: usize) {
        let start = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let items = (0..count)
            .map(|i| {
                let mut item = stac::Item::new(format!("scene-{i:08}"));
                item.collection = Some("scenes".into());
                item.properties.datetime = Some(start + chrono::Duration::minutes(i as i64));
                item.set_geometry(Some(
                    serde_json::from_value(serde_json::json!({"type":"Point","coordinates":[1,1]}))
                        .unwrap(),
                ))
                .unwrap();
                item
            })
            .collect();
        stac::geoparquet::WriterBuilder::new(File::create(path).unwrap())
            .writer_options(stac::geoparquet::WriterOptions::new().with_max_row_group_size(1024))
            .build(items)
            .unwrap()
            .finish()
            .unwrap();
    }
    fn query(count: usize) -> SearchQuery {
        let start = chrono::DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z").unwrap()
            + chrono::Duration::minutes(count as i64 - 20);
        SearchQuery {
            collections: vec!["scenes".into()],
            ids: None,
            bbox: None,
            intersects: None,
            datetime: Some(format!("{}/..", start.to_rfc3339())),
            limit: Some(20),
            sortby: None,
        }
    }
    #[tokio::test]
    async fn statistics_prune_time_groups_without_changing_results() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("items.parquet");
        fixture(&path, 4096);
        let query = query(4096);
        let search = to_stac_search(query.clone());
        let builder = parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(
            File::open(&path).unwrap(),
        )
        .unwrap();
        let groups = builder.metadata().row_groups();
        assert_eq!(groups.len(), 4);
        assert_eq!(groups.iter().filter(|g| may_match(g, &search)).count(), 1);
        let mut baseline = Vec::new();
        visit_items(&path, None, &AtomicBool::new(false), |item| {
            if search.matches(&item).unwrap() {
                baseline.push(item.id);
            }
            Ok(true)
        })
        .unwrap();
        let backend = GeoParquetBackend::new(HashMap::from([("test".into(), path)])).unwrap();
        let catalog = Catalog::new(
            "test",
            None::<String>,
            "http://127.0.0.1:9",
            None::<String>,
            None,
        )
        .unwrap();
        let result = backend
            .search(
                &catalog,
                query,
                BackendSearchOptions {
                    max_items_per_catalog: 20,
                    unify_response: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            result.iter().map(|r| r.item.id.clone()).collect::<Vec<_>>(),
            baseline
        );
    }

    /// Reproducible local benchmark; no provider access and no persistent data.
    #[tokio::test]
    #[ignore = "performance benchmark; run explicitly in release mode"]
    async fn benchmark_scoped_search() {
        let count = std::env::var("SUPERSTAC_BENCH_ITEMS")
            .ok()
            .map(|s| s.parse().unwrap())
            .unwrap_or(100_000usize);
        assert!(count >= 4096);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("items.parquet");
        fixture(&path, count);
        let bytes = std::fs::metadata(&path).unwrap().len();
        let query = query(count);
        let search = to_stac_search(query.clone());
        let backend =
            GeoParquetBackend::new(HashMap::from([("test".into(), path.clone())])).unwrap();
        let catalog = Catalog::new(
            "test",
            None::<String>,
            "http://127.0.0.1:9",
            None::<String>,
            None,
        )
        .unwrap();
        let mut baseline_ms = Vec::new();
        let mut pruned_ms = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let mut ids = Vec::new();
            visit_items(&path, None, &AtomicBool::new(false), |item| {
                if search.matches(&item).unwrap() {
                    ids.push(item.id);
                }
                Ok(ids.len() < 20)
            })
            .unwrap();
            baseline_ms.push(start.elapsed().as_secs_f64() * 1000.);
            let start = Instant::now();
            let found = backend
                .search(
                    &catalog,
                    query.clone(),
                    BackendSearchOptions {
                        max_items_per_catalog: 20,
                        unify_response: false,
                    },
                )
                .await
                .unwrap();
            pruned_ms.push(start.elapsed().as_secs_f64() * 1000.);
            assert_eq!(
                found.iter().map(|r| r.item.id.clone()).collect::<Vec<_>>(),
                ids
            );
        }
        baseline_ms.sort_by(f64::total_cmp);
        pruned_ms.sort_by(f64::total_cmp);
        println!("items={count} parquet_bytes={bytes} repeats=5 baseline_median_ms={:.3} pruned_median_ms={:.3}",baseline_ms[2],pruned_ms[2]);
    }
}
