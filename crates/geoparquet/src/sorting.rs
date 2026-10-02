use crate::error;
use serde_json::Value;
use std::cmp::Ordering;
use superstac_core::errors::SuperSTACError;
use superstac_search::response::SearchItem;
fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    fn lookup<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
        v.get(key)
            .or_else(|| key.split('.').try_fold(v, |v, part| v.get(part)))
    }
    if let Some(key) = key.strip_prefix("properties.") {
        return value
            .get("properties")
            .and_then(|p| lookup(p, key))
            .unwrap_or(&Value::Null);
    }
    lookup(value, key)
        .or_else(|| value.get("properties").and_then(|p| lookup(p, key)))
        .unwrap_or(&Value::Null)
}
fn compare(a: &Value, b: &Value) -> Ordering {
    match (a, b) {
        (Value::Null, Value::Null) => Ordering::Equal,
        (Value::Null, _) => Ordering::Greater,
        (_, Value::Null) => Ordering::Less,
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                return a.cmp(&b);
            }
            if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
                return a.cmp(&b);
            }
            a.as_f64()
                .partial_cmp(&b.as_f64())
                .unwrap_or(Ordering::Equal)
        }
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        _ => a.to_string().cmp(&b.to_string()),
    }
}
pub(crate) fn insert(
    results: &mut Vec<SearchItem>,
    item: SearchItem,
    sort: &[stac::api::Sortby],
    cap: usize,
) -> Result<(), SuperSTACError> {
    let value = serde_json::to_value(&item.item).map_err(error)?;
    let mut position = results.len();
    for (i, other) in results.iter().enumerate() {
        let other = serde_json::to_value(&other.item).map_err(error)?;
        let mut order = Ordering::Equal;
        for spec in sort {
            let a = field(&value, &spec.field);
            let b = field(&other, &spec.field);
            order = if matches!(
                spec.field.trim_start_matches("properties."),
                "datetime" | "start_datetime" | "end_datetime" | "created" | "updated"
            ) {
                match (
                    a.as_str()
                        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok()),
                    b.as_str()
                        .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok()),
                ) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    _ => compare(a, b),
                }
            } else {
                compare(a, b)
            };
            // Nulls sort last in either direction.
            if spec.direction == stac::api::Direction::Descending && !a.is_null() && !b.is_null() {
                order = order.reverse();
            }
            if order != Ordering::Equal {
                break;
            }
        }
        if order == Ordering::Equal {
            order = compare(&value["collection"], &other["collection"])
                .then_with(|| compare(&value["id"], &other["id"]));
        }
        if order == Ordering::Less {
            position = i;
            break;
        }
    }
    if position < cap {
        results.insert(position, item);
        results.truncate(cap);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_datetimes_numbers_and_nulls_sort_before_limit() {
        let mut results = Vec::new();
        for (id, date) in [
            ("fraction", "2025-01-01T00:00:00.1Z"),
            ("instant", "2025-01-01T00:00:00Z"),
        ] {
            let mut item = stac::Item::new(id);
            item.properties.datetime = Some(
                chrono::DateTime::parse_from_rfc3339(date)
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            );
            insert(
                &mut results,
                SearchItem {
                    item,
                    catalog_id: "test".into(),
                    seen_in: vec![],
                },
                &[stac::api::Sortby::asc("datetime")],
                1,
            )
            .unwrap();
        }
        assert_eq!(results[0].item.id, "instant");
        assert_eq!(
            compare(
                &serde_json::json!(9007199254740992u64),
                &serde_json::json!(9007199254740993u64)
            ),
            Ordering::Less
        );
        assert_eq!(
            compare(&Value::Null, &serde_json::json!(5)),
            Ordering::Greater
        );
        assert_eq!(
            field(
                &serde_json::json!({"id":"root","properties":{"id":"property"}}),
                "properties.id"
            ),
            "property"
        );
    }
}
