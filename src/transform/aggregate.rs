use serde::Deserialize;
use strum::EnumString;

#[derive(Deserialize, Default, Debug, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum AggregationType {
    Min,
    Max,
    Sum,
    Average,
    Count,
    #[default]
    Any,
}
