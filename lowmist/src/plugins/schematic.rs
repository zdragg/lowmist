mod mesher;

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use lowmist_schematic::Litematic;

pub struct SchematicPlugin;

impl Plugin for SchematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadedSchematic>()
            .init_asset::<SchematicAsset>()
            .init_asset_loader::<SchematicLoader>()
            .add_message::<SchematicSpawned>()
            .add_systems(Update, spawn_schematic_on_load);
    }
}

#[derive(Asset, TypePath)]
pub struct SchematicAsset(Litematic);

impl SchematicAsset {
    pub fn inner(&self) -> &Litematic {
        &self.0
    }
}

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

/// This resource holds a handle pointing at the currently loaded schematic.
///
/// The associated plugin renders the schematic stored within this resource.
///
/// To change the loaded schematic:
/// ```rust
/// fn load_schematic(
///     mut loaded_handle_resource: ResMut<LoadedSchematic>,
///     asset_server: Res<AssetServer>
/// ) {
///     let schem_asset_path = todo!();
///     loaded_handle_resource.0 = asset_server.load(schem_asset_path);
/// }
/// ```
///
/// To read the loaded schematic:
/// ```rust
/// fn read_schematic(
///     loaded_handle_resource: Res<LoadedSchematic>,
///     loaded_assets: Res<Assets<SchematicAsset>>,
/// ) {
///     let loaded_handle = &loaded_handle_resource.0;
///     if let Some(loaded_asset) = loaded_assets.get(loaded_handle) {
///         todo!();
///     }
/// }
/// ```
#[derive(Default, Resource)]
pub struct LoadedSchematic(pub Handle<SchematicAsset>);

/// A message that indicates a successful schematic spawn and includes the schemaitc bounds.
///
/// Could be used to help Cameras initialize itself with a reasonable Transform.
#[derive(Message)]
pub struct SchematicSpawned {
    /// The schematic's corner with the smallest number on each axis
    pub min_corner: IVec3,
    /// The schematic's corner with the largest number on each axis
    pub max_corner: IVec3,
}

// Do not make this component public.
// The sole purpose for this to hold the handle is for a way to check
// whether the currently spawned schematic entity and the schematic stored
// within LoadedSchematic above is the same schematic or not.
//
// If you want to get access to the schematic, do it through the LoadedSchematic Resource.
#[derive(Component)]
struct SchematicRoot(Handle<SchematicAsset>);

fn spawn_schematic_on_load(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rendered_schem: Option<Single<(Entity, &SchematicRoot)>>,
    loaded_assets: Res<Assets<SchematicAsset>>,
    loaded_handle_resource: Res<LoadedSchematic>,
    mut message_writer: MessageWriter<SchematicSpawned>,
) {
    // A "Loaded" schematic is one currently loaded within the asset server.
    // A "Rendered" schematic is the one being rendered in the world.

    let loaded_handle = &loaded_handle_resource.0;
    let Some(loaded_asset) = loaded_assets.get(loaded_handle) else {
        return; // No Loaded schematic. Nothing to do.
    };

    if let Some(rendered_schem) = rendered_schem {
        // A schematic is being Rendered.
        let (rendered_entity, rendered_root) = *rendered_schem;
        let rendered_schem_handle = &rendered_root.0; // Obtain the asset handle for the Rendered schematic.

        if rendered_schem_handle == loaded_handle {
            return; // Rendered == Loaded, nothing to do
        }

        // Otherwise, a different schematic is Loaded. Despawn the Rendered schematic entity.
        commands.entity(rendered_entity).despawn();
    }

    let schematic = &loaded_asset.0;
    let material_handle = materials.add(StandardMaterial { ..default() });

    commands
        .spawn((
            SchematicRoot(loaded_handle_resource.0.clone()), // Store the handle inside the root entity
            Transform::IDENTITY,
            Visibility::default(),
        ))
        .with_children(|parent| {
            for (chunk_pos, mesh) in mesher::build_chunked_meshes(schematic) {
                parent.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material_handle.clone()),
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
