use bevy::prelude::*;
use bevy::gltf::{Gltf, GltfNode, GltfMesh, GltfExtras};
use bevy_rapier3d::prelude::*;
use serde_json::Value;

#[derive(Default)]
struct GltfMeshes(Handle<Gltf>);

#[derive(Default)]
struct LoadedMeshes(Vec<Handle<Mesh>>);

fn main() {
	App::new()
		.insert_resource(WindowDescriptor{
			title: "Making GLTF Colliders".to_string(),
			..default()
		})
		.insert_resource(Msaa { samples: 4 })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        
		.add_plugins(DefaultPlugins)
		.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        
        .add_system(process_gltf)
        .add_system(control_extras)
        .add_startup_system(setup)

		.run();
}

fn setup(
	mut commands: Commands,	
    assets: Res<AssetServer>,
) {    
    let gltf: Handle<Gltf> = assets.load("scenes/scene1.glb");	//load gltf scene
    commands.insert_resource(GltfMeshes(gltf));
    
     commands.spawn_bundle(PointLightBundle {					//make some point light
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    
    commands.spawn_bundle(PerspectiveCameraBundle {				//make some camera
		transform: Transform::from_xyz(0.0, 26.0, -26.0).looking_at(Vec3::ZERO, Vec3::Y),
		..default()
	});
}

fn process_gltf(
	mut commands: Commands,
	mut er_gltf: EventReader<AssetEvent<Gltf>>,
	cmeshes: Res<GltfMeshes>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
){	
	for ev in er_gltf.iter() {											//iterate event reader
		if let AssetEvent::Created { handle } = ev {					//if asset created
			let scene = assets_gltf.get(handle).unwrap();				//get scene
			let mut meshes: Vec<Handle<Mesh>> = Vec::new();				//make temporary `Vec` with mesh `Handles`
			
			if *handle == cmeshes.0 {									//check whether loaded scene matches created asset
				commands.spawn_scene(scene.scenes[0].clone());			//spawn scene
				for gltfnode in scene.nodes.iter() {					//iterate nodes
					let gltfnode = assets_gltfnode.get(gltfnode);
					if let Some(gltfnode) = gltfnode {
						let mut x: Vec<Handle<Mesh>> = mesh_event(&gltfnode, &assets_gltf, &assets_gltfmesh);
						meshes.append(&mut x);							//loaded meshes to the `Vec`
					}
					
				}					
			}
			
			commands.insert_resource(LoadedMeshes(meshes));				//insert loaded meshes as `Resource`
		}
	}			
}

fn mesh_event(
	gltfnode: 			&GltfNode,
	assets_gltf: 		&Res<Assets<Gltf>>,
    assets_gltfmesh: 	&Res<Assets<GltfMesh>>,
) -> Vec<Handle<Mesh>> {
	let mut ms: Vec<Handle<Mesh>> = Vec::new();				//create temporary `Vec` of `Handles`
	
	if let Some(gltfmesh) = &gltfnode.mesh {				//... get meshes from a given node
		let gltfmesh = assets_gltfmesh.get(gltfmesh);
		if let Some(gltfmesh) = gltfmesh {
			for primitive in gltfmesh.primitives.iter() {
				let mesh = primitive.mesh.clone();
				ms.push(mesh);								//push mesh to the `Vec`
			}
		}
	}
	
	for children_node in gltfnode.children.iter() {			//recursive function call for children nodes
		ms.append(&mut mesh_event(children_node, assets_gltf, assets_gltfmesh))
	}
	
	return ms;												//return a vector
}

fn control_extras(
    mut commands: Commands,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    q_parent: Query<(Entity, &Transform, &GltfExtras), Added<GltfExtras>>,
    q_child: Query<(&Parent, Entity, &Handle<Mesh>), Added<Handle<Mesh>>>,
    loaded_meshes: Option<Res<LoadedMeshes>>,
){	
	if let Some(loaded_meshes) = loaded_meshes {									//whether `LoadedMeshes` is valid
		for (parent, ent, mesh) in q_child.iter() {									//iterate an entity
			for loaded_mesh in loaded_meshes.0.iter() {								//iterate loaded meshes
				if loaded_mesh == mesh {											//check if meshes of `LoadedMeshes` and `Entity` are equal
					if let Some(mesh) = assets_mesh.get(mesh) {						//get `Mesh` from `Handle`
						if let Some(collider) = Collider::bevy_mesh(mesh) {			//make `Collider` for `Mesh`
							for (exent, _t, gltf_extras) in q_parent.iter() {		//iterate "parent object" with `GltfExtras`
								if exent == parent.0 {								//check if parents matches
									let v: Value = serde_json::from_str(&gltf_extras.value).expect("Couldn't parse GltfExtra value as JSON");
									if v["collider"].as_str() == Some("true") {		//check whether property "collider" is true
										commands.entity(parent.0)					//add `collider` and `sensor` components to the entity
										.insert(Sensor(false))
										.insert(collider.clone())
										.insert(Ccd::enabled())						//add collision events
										.insert(ActiveCollisionTypes::default())
										.insert(ActiveEvents::COLLISION_EVENTS);
									}
								}
							}
						}
					}
				}
			}
		}
	}	
}
