use bevy::prelude::*;
use bevy::gltf::{Gltf, GltfNode, GltfMesh};
use bevy_rapier3d::prelude::*;

#[derive(Default)]
struct GltfMeshes {
	gltf: Handle<Gltf>,
	has_col: bool,
	sensor: bool,
}

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
        .add_startup_system(setup)

		.run();
}

fn setup(
	mut commands: Commands,	
    assets: Res<AssetServer>,
) {    
    let gltf: Handle<Gltf> = assets.load("scenes/scene1.glb");	//load gltf scene
    commands.insert_resource(GltfMeshes {						//create resource from the scene
		gltf,
		has_col: true,											//permits mesh collider processing
		sensor: false,											//whether the collider is sensor
	});
    
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
    assets_mesh: Res<Assets<Mesh>>,
){	
	for ev in er_gltf.iter() {										//read asset events
		if let AssetEvent::Created { handle } = ev {				//process gltf when the scene is loaded
			let scene = assets_gltf.get(handle).unwrap();
			
			if *handle == cmeshes.gltf { 							//check whether loaded scene is valid
				commands.spawn_scene(scene.scenes[0].clone());
				if cmeshes.has_col == true {						//check whether collider processing is permitted
					for gltfnode in scene.nodes.iter() {
						let gltfnode = assets_gltfnode.get(gltfnode);
						if let Some(gltfnode) = gltfnode {
							let colliders: Vec<(Collider, Transform)> = create_node_colliders(
																&gltfnode,
																&assets_gltf,
																&assets_gltfmesh,
																&assets_gltfnode,
																&assets_mesh,
															);		//create colliders from GltfMeshes with `create_node_colliders`
							
							let mut i = 0;
							while i < colliders.len() {				
								commands.spawn()
								.insert(colliders[i].0.clone())		//insert collider
								.insert(colliders[i].1.clone())		//insert GltfNode transform with scale
								.insert(Sensor(cmeshes.sensor));	//insert sensor
								i += 1;
							}
						}
						
					}
				}
			}
		}
	}
}

fn create_node_colliders(
	gltfnode: 			&GltfNode,
	assets_gltf: 		&Res<Assets<Gltf>>,
    assets_gltfmesh: 	&Res<Assets<GltfMesh>>,
    assets_gltfnode: 	&Res<Assets<GltfNode>>,
    assets_mesh: 		&Res<Assets<Mesh>>,
	
) -> Vec<(Collider, Transform)> {
	
	let mut cols: Vec<(Collider, Transform)> = Vec::new();			//create new vector
	
	if let Some(gltfmesh) = &gltfnode.mesh {						//getting all meshes from nodes
		let gltfmesh = assets_gltfmesh.get(gltfmesh);
		if let Some(gltfmesh) = gltfmesh {
			for primitive in gltfmesh.primitives.iter() {
				let mesh = assets_mesh.get(primitive.mesh.clone());
				if let Some(mesh) = mesh {
					if let Some(collider) = Collider::bevy_mesh(&mesh) { //make `bevy_mesh` colliders
						cols.push((collider, gltfnode.transform));	//push colliders and transforms to Vec `cols`
					}
				}
			}
		}
	}
	
	for children_node in gltfnode.children.iter() {					//recursive children-node processing
		let mut child_cols: Vec<(Collider, Transform)> = create_node_colliders(
														children_node,
														assets_gltf,
														assets_gltfmesh,
														assets_gltfnode,
														assets_mesh,
													);
		cols.append(&mut child_cols);
	}
	
	return cols;
}
