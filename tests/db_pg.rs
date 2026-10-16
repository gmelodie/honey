use honey::db::{row_json, rows_json};
use postgres::{Client, NoTls};
use serde_json::json;

fn client() -> Option<Client> {
    let url = std::env::var("HONEY_TEST_PG").ok()?;
    Some(Client::connect(&url, NoTls).unwrap())
}

#[test]
fn row_json_maps_supported_types_and_nulls() {
    let Some(mut c) = client() else { return };
    let r = c
        .query_one(
            "SELECT 1::int8 AS a, 2::int4 AS b, 1.5::float8 AS c, true AS d, 'x'::text AS e, 'y'::varchar AS f,
                    '2024-01-02T03:04:05Z'::timestamptz AS g, '{\"k\":[1]}'::jsonb AS h, NULL::text AS i, NULL::jsonb AS j, NULL::int8 AS k",
            &[],
        )
        .unwrap();
    assert_eq!(
        row_json(&r).unwrap(),
        json!({"a": 1, "b": 2, "c": 1.5, "d": true, "e": "x", "f": "y",
               "g": "2024-01-02T03:04:05.000000+00:00", "h": {"k": [1]}, "i": null, "j": null, "k": null})
    );
}

#[test]
fn row_json_rejects_unsupported_column_type_naming_the_column() {
    let Some(mut c) = client() else { return };
    let r = c.query_one("SELECT 1::int2 AS small", &[]).unwrap();
    let e = row_json(&r).unwrap_err().to_string();
    assert!(e.contains("small") && e.contains("unsupported type"), "{e}");
}

#[test]
fn rows_json_empty_result_is_empty_vec() {
    let Some(mut c) = client() else { return };
    let rows = c.query("SELECT 1 WHERE false", &[]).unwrap();
    assert!(rows_json(&rows).unwrap().is_empty());
}
