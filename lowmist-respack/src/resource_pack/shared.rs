use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(try_from = "i32")]
pub enum QuarterRotation {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl TryFrom<i32> for QuarterRotation {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value.rem_euclid(360) {
            0 => Ok(Self::R0),
            90 => Ok(Self::R90),
            180 => Ok(Self::R180),
            270 => Ok(Self::R270),
            _ => Err("invalid rotation".to_owned()),
        }
    }
}
