# GLTF scene collider example (bevy_rapier3d)

**A conception of the example was changed.** All components are unified for comfortable use. Each Entity of the loaded scene has the following structure:

* Parent
	* Transform
	* GltfExtras

* Child
	* Handle\<Mesh\>
	
To add a collider to a Mesh of the scene, it must have extras `collider = true`

**The whole code is licensed under Apache 2.0 license**

## Dependencies

	bevy = "0.7"
	bevy_rapier3d = "0.14.1", features=["debug-render"]
	serde_json = "1.0"
