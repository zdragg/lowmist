pub(crate) mod ivec3 {
    pub(crate) mod xyz {
        use glam::IVec3;
        use serde::{Deserialize, Deserializer};

        #[derive(Deserialize)]
        struct Xyz {
            x: i32,
            y: i32,
            z: i32,
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<IVec3, D::Error> {
            let Xyz { x, y, z } = Xyz::deserialize(de)?;
            Ok(IVec3 { x, y, z })
        }
    }
}
