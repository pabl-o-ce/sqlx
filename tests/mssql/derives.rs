use sqlx::mssql::Mssql;
use sqlx_test::{new, test_type};

#[sqlx::test]
async fn test_derive_weak_enum() -> anyhow::Result<()> {
    #[derive(sqlx::Type, Debug, PartialEq, Eq)]
    #[repr(i16)]
    enum WeakEnumI16 {
        Foo = i16::MIN,
        Bar = 0,
        Baz = i16::MAX,
    }

    #[derive(sqlx::Type, Debug, PartialEq, Eq)]
    #[repr(i32)]
    enum WeakEnumI32 {
        Foo = i32::MIN,
        Bar = 0,
        Baz = i32::MAX,
    }

    #[derive(sqlx::Type, Debug, PartialEq, Eq)]
    #[repr(i64)]
    enum WeakEnumI64 {
        Foo = i64::MIN,
        Bar = 0,
        Baz = i64::MAX,
    }

    #[derive(sqlx::FromRow, Debug, PartialEq, Eq)]
    struct WeakEnumRow {
        i16: WeakEnumI16,
        i32: WeakEnumI32,
        i64: WeakEnumI64,
    }

    let mut conn = new::<Mssql>().await?;

    sqlx::raw_sql(
        r#"
            CREATE TABLE #weak_enum (
                i16 SMALLINT,
                i32 INT,
                i64 BIGINT
            )
        "#,
    )
    .execute(&mut conn)
    .await?;

    let rows_in = vec![
        WeakEnumRow {
            i16: WeakEnumI16::Foo,
            i32: WeakEnumI32::Foo,
            i64: WeakEnumI64::Foo,
        },
        WeakEnumRow {
            i16: WeakEnumI16::Bar,
            i32: WeakEnumI32::Bar,
            i64: WeakEnumI64::Bar,
        },
        WeakEnumRow {
            i16: WeakEnumI16::Baz,
            i32: WeakEnumI32::Baz,
            i64: WeakEnumI64::Baz,
        },
    ];

    for row in &rows_in {
        sqlx::query(
            r#"
                INSERT INTO #weak_enum(i16, i32, i64)
                VALUES (@p1, @p2, @p3)
            "#,
        )
        .bind(&row.i16)
        .bind(&row.i32)
        .bind(&row.i64)
        .execute(&mut conn)
        .await?;
    }

    let rows_out: Vec<WeakEnumRow> = sqlx::query_as("SELECT * FROM #weak_enum")
        .fetch_all(&mut conn)
        .await?;

    assert_eq!(rows_in, rows_out);

    Ok(())
}

#[derive(PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct TransparentTuple(i64);

#[derive(PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(transparent)]
struct TransparentNamed {
    field: i64,
}

test_type!(transparent_tuple<TransparentTuple>(Mssql,
    "CAST(0 AS BIGINT)" == TransparentTuple(0),
    "CAST(23523 AS BIGINT)" == TransparentTuple(23523)
));

test_type!(transparent_named<TransparentNamed>(Mssql,
    "CAST(0 AS BIGINT)" == TransparentNamed { field: 0 },
    "CAST(23523 AS BIGINT)" == TransparentNamed { field: 23523 },
));
