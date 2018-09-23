
// use amethyst_assets;
use amethyst_assets::{
//    AssetStorage, Error, ErrorKind, Format, Handle, Loader, PrefabData, PrefabError,
    ProcessingState, 
    // ProgressCounter, 
    Result as AResult, 
    // ResultExt,
         SimpleFormat,
};

// use amethyst_core::specs::prelude::{Entity, Read, ReadExpect};
use Renderer;
use Program;
use ProgramData;

/// Allow loading of glsl shaders
#[derive(Clone)]
pub struct GlslProgram;

impl SimpleFormat<Program> for GlslProgram {
    const NAME: &'static str = "GlslProgram";

    type Options = ();

    fn import(&self, bytes: Vec<u8>, _: ()) -> AResult<Box<Vec<u8>>> {
        Ok(Box::new(bytes))
        // from_str(s).chain_err(|| "Failed to decode mesh file")
    }
}


/// Create a texture asset.
pub fn create_shader_asset(
    data: ProgramData,
    renderer: &mut Renderer,
) -> AResult<ProcessingState<Program>> {
    println!("process {}", data.len());
    renderer.program_loaded();
    Ok(ProcessingState::Loaded(Program::new(data)))
}

// #[cfg(test)]
// mod tests {
//     use super::TextureData;

//     #[test]
//     fn texture_data_from_f32_3() {
//         match TextureData::from([0.25, 0.50, 0.75]) {
//             TextureData::Rgba(color, _) => {
//                 assert_eq!(color, [0.25, 0.50, 0.75, 1.0]);
//             }
//             _ => panic!("Expected [f32; 3] to turn into TextureData::Rgba"),
//         }
//     }
// }
