//! Simple shaded pass

use super::*;
use amethyst_assets::{AssetStorage, Loader, ProgressCounter};
use amethyst_core::specs::prelude::{Join, Read, ReadExpect, ReadStorage};
use amethyst_core::transform::GlobalTransform;
use cam::{ActiveCamera, Camera};
use error::Result;
use formats::GlslProgram;
use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};
use light::Light;
use mesh::{Mesh, MeshHandle};
use mtl::{Material, MaterialDefaults};
use pass::shaded_util::{set_light_args, setup_light_buffers};
use pass::util::{draw_mesh, get_camera, setup_textures, setup_vertex_args};
use pipe::Init;
use pipe::pass::{Pass, PassData};
use pipe::{DepthMode, Effect, EffectBuilder, NewEffect, Program, ProgramHandle, ProgramSource};
use resources::AmbientColor;
use std::marker::PhantomData;
use tex::Texture;
use types::{Encoder, Factory};
use vertex::{Normal, Position, Query, TexCoord};
use visibility::Visibility;

/// Draw mesh with simple lighting technique
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Normal, TexCoord)>"), Clone(bound=""))]
pub struct DrawShaded<V> {
    _pd: PhantomData<V>,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl<V> DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    /// Create instance of `DrawShaded` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// asd
    pub fn new_with_program(p: ProgramSource) -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a, V> PassData<'a> for DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AmbientColor>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        Read<'a, AssetStorage<Program>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, Material>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Light>,
    );
}

impl<V> Pass for DrawShaded<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    fn get_init<'a>(&self, mut builder: EffectBuilder<'a>) -> Option<Init<'a>> {
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        Some(builder.init.clone())
    }
    fn compile<'a>(&mut self, effect: NewEffect<'a>) -> EffectBuilder<'a> {
        // unreachable!()
        // let mut progress = ProgressCounter::new();
        let vs_handle = effect.loader.load(
            "shader/vertex.glsl",
            GlslProgram,
            (),
            (),
            effect.storage,
        );
        let ps_handle = effect.loader.load(
            "shader/frag.glsl",
            GlslProgram,
            (),
            (),
            effect.storage,
        );
        let mut builder = effect.simple_handles(vs_handle, ps_handle);
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_textures(&mut builder, &TEXTURES);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            active,
            camera,
            ambient,
            mesh_storage,
            tex_storage,
            program_storage,
            material_defaults,
            visibility,
            mesh,
            material,
            global,
            light,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        set_light_args(effect, encoder, &light, &global, &ambient, camera);

        match visibility {
            None => for (mesh, material, global) in (&mesh, &material, &global).join() {
                draw_mesh(
                    encoder,
                    effect,
                    false,
                    mesh_storage.get(mesh),
                    None,
                    &tex_storage,
                    Some(material),
                    &material_defaults,
                    camera,
                    Some(global),
                    &[V::QUERIED_ATTRIBUTES],
                    &TEXTURES,
                );
            },
            Some(ref visibility) => {
                for (mesh, material, global, _) in
                    (&mesh, &material, &global, &visibility.visible_unordered).join()
                {
                    draw_mesh(
                        encoder,
                        effect,
                        false,
                        mesh_storage.get(mesh),
                        None,
                        &tex_storage,
                        Some(material),
                        &material_defaults,
                        camera,
                        Some(global),
                        &[V::QUERIED_ATTRIBUTES],
                        &TEXTURES,
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(mesh) = mesh.get(*entity) {
                        draw_mesh(
                            encoder,
                            effect,
                            false,
                            mesh_storage.get(mesh),
                            None,
                            &tex_storage,
                            material.get(*entity),
                            &material_defaults,
                            camera,
                            global.get(*entity),
                            &[V::QUERIED_ATTRIBUTES],
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}
