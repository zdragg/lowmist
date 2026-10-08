mod mesher;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use lowmist_schematic::Litematic;

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SchematicHandle>()
            .init_asset::<SchematicAsset>()
            .init_asset_loader::<SchematicLoader>()
            .add_message::<SchematicSpawned>()
            .add_systems(Update, spawn_schematic_on_load);
    }
}

#[derive(Asset, TypePath)]
pub struct SchematicAsset(Litematic);

#[derive(Default, Resource)]
pub struct SchematicHandle(pub Handle<SchematicAsset>);

#[derive(Default, TypePath)]
struct SchematicLoader;

impl AssetLoader for SchematicLoader {
    type Asset = SchematicAsset;
    type Settings = ();
    type Error = BevyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let schematic = Litematic::parse(&bytes[..])?;
        Ok(SchematicAsset(schematic))
    }

    fn extensions(&self) -> &[&str] {
        &["litematic"]
    }
}

/// A message that indicates a successful schematic spawn and includes the schemaitc bounds.
///
/// Use this to initialize the camera with reasonable numbers.
#[derive(Message)]
pub struct SchematicSpawned {
    /// The schematic's corner with the smallest number on each axis
    pub min_corner: IVec3,
    /// The schematic's corner with the largest number on each axis
    pub max_corner: IVec3,
}

#[derive(Component)]
struct SchematicRoot(Handle<SchematicAsset>);

fn spawn_schematic_on_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut message_writer: MessageWriter<SchematicSpawned>,
    loaded_schem: Option<Single<(Entity, &SchematicRoot)>>,

    schematics: Res<Assets<SchematicAsset>>,
    asset_handle: Res<SchematicHandle>,
) {
    let Some(asset_schem) = schematics.get(&asset_handle.0) else {
        return; // No schematic loaded
    };
    // A schematic exists in asset storage.

    // If a schematic is already loaded:
    if let Some(loaded_schem) = loaded_schem {
        let (loaded_entity, loaded_parent) = *loaded_schem;
        let loaded_handle = &loaded_parent.0; // Obtain the handle stored in SchematicRoot

        if loaded_handle == &asset_handle.0 {
            // Loaded schematic is equal to the one in storage.
            return;
        }

        // Otherwise, a new schematic has loaded. Despawn the currently loaded entity.
        commands.entity(loaded_entity).despawn();
    }

    let schematic = &asset_schem.0;
    let standard_material = materials.add(StandardMaterial { ..default() });

    commands
        .spawn((
            SchematicRoot(asset_handle.0.clone()), // Store the handle inside the root entity
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .with_children(|parent| {
            for (chunk_pos, mesh) in mesher::build_chunked_meshes(schematic) {
                parent.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(standard_material.clone()),
                    Transform::from_translation((chunk_pos * 16).as_vec3()),
                ));
            }
        });

    let (min_corner, max_corner) = schematic.bounds();

    // Broadcast schematic spawn along with schematic bounds
    message_writer.write(SchematicSpawned {
        min_corner,
        max_corner,
    });
}
