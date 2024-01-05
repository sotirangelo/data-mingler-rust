pub mod aggregate;

use serde::Deserialize;
use strum::EnumString;

#[derive(Deserialize, Debug, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Transformation {
    Aggregate(aggregate::AggregationType),
    Filter(String),
    Map(String),
}
