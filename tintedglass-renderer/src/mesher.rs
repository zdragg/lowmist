use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use tintedglass_model::Litematic;

pub(super) fn build_mesh(schematic: &Litematic) -> Mesh {
    // BlockId(num) -> color_map(num) to find color
    let color_map: Vec<_> = schematic
        .palettes()
        .map(|(_, palette)| {
            let mut hasher = DefaultHasher::new();
            palette.name.hash(&mut hasher);
            let hue = (hasher.finish() % 360) as f32;
            Color::hsl(hue, 0.6, 0.5)
        })
        .collect();

    // Corner coordinates
    let mut positions = vec![];
    // Face directions
    let mut normals = vec![];
    // Colors
    let mut colors = vec![];
    // Total face count
    let mut total_face_count = 0;

    for (pos, block_id) in schematic.blocks_without_air() {
        if block_id.is_air() {
            continue;
        }

        for face in Face::FACES {
            let neighbor_pos = pos + face.neighbor_offset();
            if let Some(block_id) = schematic.block_at(neighbor_pos)
                && !block_id.is_air()
                && schematic.palette_entry(block_id).is_opaque()
            {
                // The face is next to an opaque block, culled
                continue;
            }

            // Append the 4 global corner positions of this face, 3 [f32; 4] per face
            positions.extend(face.corners().into_iter().map(|rel_corner_pos| {
                let corner_pos = rel_corner_pos + pos.as_vec3();
                corner_pos.to_array()
            }));

            // Append the normal of the face, once per vertex, 3 [f32; 4] per face
            normals.extend(std::iter::repeat_n(face.normal().to_array(), 4));

            // Append the color of the face, once per vertex, 4 [f32; 4] per face
            colors.extend(std::iter::repeat_n(
                color_map[block_id.id() as usize].to_linear().to_f32_array(),
                4,
            ));
            total_face_count += 1;
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(indices(total_face_count))
}

/// Returns the indices vector for `count` faces.
fn indices(count: u32) -> Indices {
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
