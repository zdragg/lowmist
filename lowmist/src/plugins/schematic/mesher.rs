use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::{collections::HashMap, hash::fixed_hash_one},
    prelude::*,
};
use lowmist_schematic::Litematic;

pub(super) fn build_chunked_meshes(schematic: &Litematic) -> impl Iterator<Item = (IVec3, Mesh)> {
    // BlockId(num) -> color_map(num) to find color
    let color_map: Vec<_> = schematic
        .palettes()
        .map(|(_, palette)| {
            let hue = (fixed_hash_one(&palette.name) % 360) as f32;
            Color::hsl(hue, 0.6, 0.5)
        })
        .collect();

    #[derive(Default)]
    struct Attributes {
        // Corner coordinates
        positions: Vec<[f32; 3]>,
        // Face directions
        normals: Vec<[f32; 3]>,
        // Colors
        colors: Vec<[f32; 4]>,
        // Total face count
        face_count: u32,
    }

    let mut attributes: HashMap<IVec3, Attributes> = HashMap::new();

    for (pos, block_id) in schematic.blocks_without_air() {
        let chunk_pos = pos.div_euclid(IVec3::splat(16));
        let rel_chunk_pos = pos.rem_euclid(IVec3::splat(16));
        let chunk_attributes = attributes.entry(chunk_pos).or_default();

        for face in Face::FACES {
            let neighbor_pos = pos + face.neighbor_offset();
            if let Some(neighbor_id) = schematic.block_at(neighbor_pos)
                && !neighbor_id.is_air()
                && schematic.palette_entry(neighbor_id).is_opaque()
            {
                // The face is next to an opaque block, culled
                continue;
            }

            // Append the 4 corner positions of this face, 4 [f32; 3] per face
            chunk_attributes
                .positions
                .extend(face.corners().into_iter().map(|rel_corner_pos| {
                    let corner_pos = rel_corner_pos + rel_chunk_pos.as_vec3();
                    corner_pos.to_array()
                }));

            // Append the normal of the face, once per vertex, 4 [f32; 3] per face
            chunk_attributes
                .normals
                .extend(std::iter::repeat_n(face.normal().to_array(), 4));

            // Append the color of the face, once per vertex, 4 [f32; 4] per face
            chunk_attributes.colors.extend(std::iter::repeat_n(
                color_map[block_id.id() as usize].to_linear().to_f32_array(),
                4,
            ));

            // increment face count
            chunk_attributes.face_count += 1;
        }
    }

    attributes
        .into_iter()
        .filter_map(|(chunk_pos, attributes)| {
            if attributes.face_count == 0 {
                return None;
            }
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, attributes.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attributes.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, attributes.colors)
            .with_inserted_indices(indices(attributes.face_count));
            Some((chunk_pos, mesh))
        })
}

/// Returns the indices vector for `count` faces.
fn indices(count: u32) -> Indices {
    if count <= 16384 {
        // Indices fits within u16
        let mut vec = Vec::with_capacity(6 * count as usize);
        for face in 0..count as u16 {
            let offset = face * 4;
            vec.extend_from_slice(&[
                offset,
                offset + 1,
                offset + 2,
                offset,
                offset + 2,
                offset + 3,
            ]);
        }
        Indices::U16(vec)
    } else {
        let mut vec = Vec::with_capacity(6 * count as usize);
        for face in 0..count {
            let offset = face * 4;
            vec.extend_from_slice(&[
                offset,
                offset + 1,
                offset + 2,
                offset,
                offset + 2,
                offset + 3,
            ]);
        }
        Indices::U32(vec)
    }
}

struct Face {
    corners: [Vec3; 4],
    direction: IVec3,
}

impl Face {
    /// The four corners in counter-clockwise order
    fn corners(&self) -> &[Vec3; 4] {
        &self.corners
    }
    /// The direction that the face points at
    fn normal(&self) -> Vec3 {
        self.direction.as_vec3()
    }
    /// Relative position of the block that the face touches
    fn neighbor_offset(&self) -> IVec3 {
        self.direction
    }

    // Use 1x1x1 cube on Desmos 3D to visualize:
    // 1. Find the corresponding face
    // For +x, start from the center point of the cube and point toward +x.
    // The face it passes through is the +x face.
    // 2. Label each vertex
    // Start from any point (I started from the point touching the axis), then go around
    // counter-clockwise. Label each point in order: 0, 1, 2, 3.
    // 3. Record the coordinates for each point and put them into the vector in that order.
    // Then, for each face, the triangles (0, 1, 2) and (0, 2, 3) make up the visible face.

    const FACES: [Face; 6] = [
        Face::Y,
        Face::NEG_Y,
        Face::X,
        Face::NEG_X,
        Face::Z,
        Face::NEG_Z,
    ];

    const Y: Face = Face {
        corners: [
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 1.0, 0.0),
        ],
        direction: IVec3::Y,
    };

    const NEG_Y: Face = Face {
        corners: [
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
        ],
        direction: IVec3::NEG_Y,
    };

    const X: Face = Face {
        corners: [
            vec3(1.0, 0.0, 0.0),
            vec3(1.0, 1.0, 0.0),
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 0.0, 1.0),
        ],
        direction: IVec3::X,
    };

    const NEG_X: Face = Face {
        corners: [
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 1.0, 1.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 0.0, 0.0),
        ],
        direction: IVec3::NEG_X,
    };

    const Z: Face = Face {
        corners: [
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 0.0, 1.0),
            vec3(1.0, 1.0, 1.0),
            vec3(0.0, 1.0, 1.0),
        ],
        direction: IVec3::Z,
    };

    const NEG_Z: Face = Face {
        corners: [
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(1.0, 1.0, 0.0),
            vec3(1.0, 0.0, 0.0),
        ],
        direction: IVec3::NEG_Z,
    };
}
