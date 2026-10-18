use serde::Deserialize;

use crate::resource_pack::ResourceLocation;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub parent: Option<ResourceLocation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_template_torch() {
        let json = r##"
        {
           "ambientocclusion": false,
           "textures": {
               "particle": "#torch"
           },
           "elements": [
               {   "from": [ 7, 0, 7 ],
                   "to": [ 9, 10, 9 ],
                   "shade": false,
                   "faces": {
                       "down": { "uv": [ 7, 13, 9, 15 ], "texture": "#torch" },
                       "up":   { "uv": [ 7,  6, 9,  8 ], "texture": "#torch" }
                   }
               },
               {   "from": [ 7, 0, 0 ],
                   "to": [ 9, 16, 16 ],
                   "shade": false,
                   "faces": {
                       "west": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                       "east": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
                   }
               },
               {   "from": [ 0, 0, 7 ],
                   "to": [ 16, 16, 9 ],
                   "shade": false,
                   "faces": {
                       "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                       "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
                   }
               }
           ]
        }
        "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_any_block() {
        let json = r##"
            {
              "elements": [
                {
                  "from": [ 0, 0, 0 ],
                  "to": [ 16, 16, 16 ],
                  "faces": {
                    "down":  { "texture": "#down", "cullface": "down" },
                    "up":    { "texture": "#up", "cullface": "up" },
                    "north": { "texture": "#north", "cullface": "north" },
                    "south": { "texture": "#south", "cullface": "south" },
                    "west":  { "texture": "#west", "cullface": "west" },
                    "east":  { "texture": "#east", "cullface": "east" }
                  }
                }
              ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_sapling() {
        let json = r##"
            {
              "ambientocclusion": false,
              "textures": {
                "particle": "#cross"
              },
              "elements": [
                {
                  "from": [ 0.8, 0, 8 ],
                  "to": [ 15.2, 16, 8 ],
                  "rotation": { "origin": [ 8, 8, 8 ], "axis": "y", "angle": 45, "rescale": true },
                  "shade": false,
                  "faces": {
                    "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" },
                    "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" }
                  }
                },
                {
                  "from": [ 8, 0, 0.8 ],
                  "to": [ 8, 16, 15.2 ],
                  "rotation": { "origin": [ 8, 8, 8 ], "axis": "y", "angle": 45, "rescale": true },
                  "shade": false,
                  "faces": {
                    "west": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" },
                    "east": { "uv": [ 0, 0, 16, 16 ], "texture": "#cross" }
                  }
                }
              ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn parse_fletching_table() {
        let json = r##"
            {
                "parent": "block/cube",
                "textures": {
                    "particle": "block/fletching_table",
                    "all": "block/fletching_table",
                    "bottom": "block/birch_planks"
                },
                "elements": [
                    {
                       "from": [ 0, 0, 0 ],
                        "to": [ 16, 16, 16 ],
                        "faces": {
                            "down":  { "texture": "#bottom", "cullface": "down" },
                            "up":    { "uv": [0,0, 16,5.34], "texture": "#all", "cullface": "up" },
                            "north": { "uv": [0,5.35, 16,10.66], "texture": "#all", "cullface": "north" },
                            "south": { "uv": [0,5.35, 16,10.66], "texture": "#all", "cullface": "south" },
                            "west":  { "uv": [0,10.67, 16,16], "texture": "#all", "cullface": "west" },
                            "east":  { "uv": [0,10.67, 16,16], "texture": "#all", "cullface": "east" }
                        }
                    }
                ]
            }
            "##;

        let _: Model = serde_json::from_str(json).unwrap();
    }
}
