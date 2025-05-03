use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use duckdb::{
    Connection, params,
    types::{FromSql, FromSqlError, FromSqlResult, TimeUnit, ValueRef},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl FromSql for Timestamp {
    #[inline]
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            // Timestamp stored as microseconds or milliseconds since epoch
            ValueRef::Timestamp(unit, i) => {
                let timestamp = match unit {
                    TimeUnit::Microsecond => DateTime::from_timestamp_micros(i),
                    TimeUnit::Millisecond => DateTime::from_timestamp_millis(i),
                    TimeUnit::Nanosecond => Some(DateTime::from_timestamp_nanos(i)),
                    _ => {
                        return Err(FromSqlError::Other(
                            "Unsupported TimeUnit for Timestamp".to_string().into(),
                        ));
                    }
                };

                if let Some(timestamp) = timestamp {
                    Ok(Timestamp(timestamp))
                } else {
                    Err(FromSqlError::Other(
                        "Unsupported TimeUnit for Timestamp".to_string().into(),
                    ))
                }
            }

            // Date stored as days since Unix epoch
            ValueRef::Date32(i) => {
                if let Some(naive_date) = NaiveDate::from_num_days_from_ce_opt(i + 719_163) {
                    if let Some(naive_date_time) = naive_date.and_hms_opt(0, 0, 0) {
                        Ok(Timestamp(DateTime::from_naive_utc_and_offset(
                            naive_date_time,
                            Utc,
                        )))
                    } else {
                        Err(FromSqlError::Other(
                            "Invalid time from date".to_string().into(),
                        ))
                    }
                } else {
                    Err(FromSqlError::Other("Invalid date value".to_string().into()))
                }
            }
            ValueRef::Time64(TimeUnit::Microsecond, i) => {
                let secs = (i / 1_000_000) as u32;
                let nsecs = ((i % 1_000_000) * 1_000) as u32;
                match NaiveTime::from_num_seconds_from_midnight_opt(secs, nsecs) {
                    Some(naive_time) => match NaiveDate::from_ymd_opt(1970, 1, 1) {
                        Some(naive_date) => Ok(Timestamp(DateTime::from_naive_utc_and_offset(
                            naive_date.and_time(naive_time),
                            Utc,
                        ))),
                        _ => Err(FromSqlError::Other("Invalid time".to_string().into())),
                    },
                    _ => Err(FromSqlError::Other("Invalid time".to_string().into())),
                }
            }

            _ => Err(FromSqlError::Other(
                "Unsupported value type for Timestamp".to_string().into(),
            )),
        }
    }
}

// impl FromSql for MyDateTime {
//     #[inline]
//     fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
//         match value {
//             // Timestamp stored as microseconds or milliseconds since epoch
//             ValueRef::Timestamp(unit, i) => {
//                 let (secs, nsecs) = match unit {
//                     TimeUnit::Microsecond => (i / 1_000_000, ((i % 1_000_000) * 1_000) as u32),
//                     TimeUnit::Millisecond => (i / 1_000, ((i % 1_000) * 1_000_000) as u32),
//                     _ => {
//                         return Err(FromSqlError::Other(
//                             "Unsupported TimeUnit for Timestamp".to_string().into(),
//                         ));
//                     }
//                 };
//                 let date_time = DateTime::from_timestamp(secs, nsecs).unwrap();
//                 Ok(MyDateTime(date_time))
//                 // let naive = NaiveDateTime::from_timestamp(secs, nsecs);
//                 // Ok(MyDateTime(DateTime::<Utc>::from_utc(naive, Utc)))
//             }

//             // Date stored as days since Unix epoch
//             ValueRef::Date32(i) => {
//                 if let Some(naive_date) = NaiveDate::from_num_days_from_ce_opt(i + 719_163) {
//                     if let Some(naive_date_time) = naive_date.and_hms_opt(0, 0, 0) {
//                         Ok(MyDateTime(DateTime::from_naive_utc_and_offset(
//                             naive_date_time,
//                             Utc,
//                         )))
//                     } else {
//                         Err(FromSqlError::Other(
//                             "Invalid time from date".to_string().into(),
//                         ))
//                     }
//                 } else {
//                     Err(FromSqlError::Other("Invalid date value".to_string().into()))
//                 }
//             }
//             ValueRef::Time64(TimeUnit::Microsecond, i) => {
//                 let secs = (i / 1_000_000) as u32;
//                 let nsecs = ((i % 1_000_000) * 1_000) as u32;
//                 match NaiveTime::from_num_seconds_from_midnight_opt(secs, nsecs) {
//                     Some(naive_time) => match NaiveDate::from_ymd_opt(1970, 1, 1) {
//                         Some(naive_date) => Ok(MyDateTime(DateTime::from_naive_utc_and_offset(
//                             naive_date.and_time(naive_time),
//                             Utc,
//                         ))),
//                         _ => Err(FromSqlError::Other("Invalid time".to_string().into())),
//                     },
//                     _ => Err(FromSqlError::Other("Invalid time".to_string().into())),
//                 }
//             }

//             _ => Err(FromSqlError::Other(
//                 "Unsupported value type for MyDateTime".to_string().into(),
//             )),
//         }
//     }
// }

// let dt = match unit {
//     duckdb::types::TimeUnit::Nanosecond => DateTime::from_timestamp_nanos(amount),
//     duckdb::types::TimeUnit::Microsecond => {
//         DateTime::from_timestamp_micros(amount).ok_or(error)?
//     }
//     duckdb::types::TimeUnit::Millisecond => {
//         DateTime::from_timestamp_millis(amount).ok_or(error)?
//     }
//     duckdb::types::TimeUnit::Second => {
//         DateTime::from_timestamp(amount, 0).ok_or(error)?
//     }
// };
