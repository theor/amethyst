// //! Defining a custom asset and format.

// extern crate amethyst_assets;
// extern crate amethyst_core;
// extern crate amethyst_renderer;
// extern crate rayon;

// use amethyst_assets::*;
// use amethyst_core::specs::prelude::VecStorage;
// use amethyst_renderer::{Program,ProgramHandle, GlslProgram};
// use rayon::ThreadPoolBuilder;
// use std::str::from_utf8;
// use std::sync::Arc;
// use std::thread::sleep;
// use std::time::Duration;

// fn main() {
//     let path = format!("{}/examples/assets", env!("CARGO_MANIFEST_DIR"));

//     let builder = ThreadPoolBuilder::new().num_threads(8);
//     let pool = Arc::new(builder.build().expect("Invalid config"));

//     let loader = Loader::new(&path, pool.clone());
//     let mut storage = AssetStorage::new();

//     let mut progress = ProgressCounter::new();

//     let dummy = loader.load(
//         "shader/vertex.glsl",
//         GlslProgram,
//         (),
//         &mut progress,
//         &storage,
//     );
//     println!("dummy: {:?}", storage.get(&dummy));

//     // Hot-reload every three seconds.
//     let strategy = HotReloadStrategy::every(3);

//     // Game loop
//     let mut frame_number = 0;
//     println!("start");

//     loop {
//         frame_number += 1;

//         // If loading is done, end the game loop and print the asset
//         if progress.is_complete() {
//             println!("done");
//         }

//         // Do per-frame stuff (display loading screen, ..)
//         sleep(Duration::new(1, 0));

//         storage.process(
//             |mut s| {
//                 println!("process {}", s.len());
//                 Ok(ProcessingState::Loaded(Program::new(s)))
//             },
//             frame_number,
//             &*pool,
//             Some(&strategy),
//         );
//     }

//     println!("dummy: {:?}", storage.get(&dummy));
// }

//! Displays a shaded sphere to the user.

extern crate amethyst;

use amethyst::assets::{PrefabLoader, PrefabLoaderSystem, RonFormat, HotReloadStrategy, HotReloadBundle};
use amethyst::core::transform::TransformBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DrawFlat, DrawShaded, PosNormTex};
use amethyst::utils::application_root_dir;
use amethyst::utils::scene::BasicScenePrefab;

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example;

impl<'a, 'b> SimpleState<'a, 'b> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Initialise the scene with an object, a light and a camera.
        let handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load("prefab/sphere.ron", RonFormat, (), ())
        });
        data.world.create_entity().with(handle).build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = ".";// application_root_dir();

    let display_config_path = format!("{}/examples/sphere/resources/display_config.ron", app_root);

    let resources = format!("{}/examples/assets/", app_root);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(HotReloadBundle::new(HotReloadStrategy::every(1)))?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), false)?;
        // .with_basic_renderer(display_config_path, DrawFlat::<PosNormTex>::new(), false)?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}