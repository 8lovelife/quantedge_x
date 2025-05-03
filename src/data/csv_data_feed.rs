use super::data_feed::DataFeed;
use crate::strategy::market_data::MarketData;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;

pub struct CsvDataFeed {
    pub records: Vec<MarketData>,
    pub cursor: usize,
}

impl CsvDataFeed {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(file);

        #[derive(serde::Deserialize)]
        struct RawRecord {
            #[serde(rename = "Start")]
            start: String,
            #[serde(rename = "Open")]
            open: f64,
            #[serde(rename = "High")]
            high: f64,
            #[serde(rename = "Low")]
            low: f64,
            #[serde(rename = "Close")]
            close: f64,
        }

        let mut records = Vec::new();
        for result in rdr.deserialize() {
            let raw: RawRecord = result?;
            records.push(MarketData {
                timestamp: raw.start,
                close_price: raw.close,
            });
        }

        Ok(Self { records, cursor: 0 })
    }
}

impl DataFeed for CsvDataFeed {
    fn next(&mut self) -> Option<MarketData> {
        if self.cursor < self.records.len() {
            let rec = self.records[self.cursor].clone();
            self.cursor += 1;
            Some(rec)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }
}
