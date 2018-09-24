//! Types for constructing render passes.

use amethyst_core::specs::prelude::SystemData;
use amethyst_assets::{AssetStorage, Loader};

use error::Result;
use pipe::{Effect, NewEffect, Program, Target, Init, EffectBuilder};
use types::{Encoder, Factory};
use super::super::Renderer;

/// Used to fetch data from the game world for rendering in the pass.
pub trait PassData<'a> {
    /// The data itself.
    type Data: SystemData<'a> + Send;
}

/// Structures implementing this provide a renderer pass.
pub trait Pass: Clone + for<'a> PassData<'a> {
    /// TODO Init
    fn get_init<'a>(&self, builder: EffectBuilder<'a>) -> Option<Init<'a>> {
        None
    }
    /// The pass is given an opportunity to compile shaders and store them in an `Effect`
    /// which is then passed to the pass in `apply`.
    fn compile<'a>(&mut self, effect: NewEffect<'a>) -> EffectBuilder<'a>;
    /// Called whenever the renderer is ready to apply the pass.  Feed commands into the
    /// encoder here.
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        factory: Factory,
        data: <Self as PassData<'b>>::Data,
    );
}

/// A compiled pass.  These are created and managed by the `Renderer`.  This should not be
/// used directly outside of the renderer.
#[derive(Clone, Debug)]
pub struct CompiledPass<P> {
    effect: Effect,
    inner: P,
}

impl<P> CompiledPass<P>
where
    P: Pass,
{
    pub(super) fn compile(
        mut pass: P,
        renderer: &mut Renderer,
        out: &Target,
        multisampling: u16,
        loader: &Loader,
        storage: &AssetStorage<Program>,
    ) -> Result<Self> {
        let mut effect = pass.compile(NewEffect::new(loader, storage));
        let effect = effect.build(renderer, storage, out, multisampling)?;
        Ok(CompiledPass {
            effect,
            inner: pass,
        })
    }
}

impl<P> CompiledPass<P> {
    /// Reload the inner pass
    pub fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
    ) where
        P: Pass,
    {
        // use pipe::ProgramSource;
        // let p = ProgramSource::SimpleHandle(self.effect.prog[0], self.effect.prog[1]);
        let i = self.inner.get_init(EffectBuilder::new(self.effect.program())).unwrap();
        self.effect.reload(renderer, storage, i);
    }
    /// Applies the inner pass.
    pub fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <P as PassData<'b>>::Data,
    ) where
        P: Pass,
    {
        self.inner.apply(encoder, &mut self.effect, factory, data)
    }

    /// Distributes new target data to the pass.
    pub fn new_target(&mut self, target: &Target) {
        // Distribute new targets that don't blend.
        self.effect.data.out_colors.clear();
        self.effect
            .data
            .out_colors
            .extend(target.color_bufs().iter().map(|cb| &cb.as_output).cloned());

        // Distribute new blend targets
        self.effect.data.out_blends.clear();
        self.effect
            .data
            .out_blends
            .extend(target.color_bufs().iter().map(|cb| &cb.as_output).cloned());

        // Distribute new depth buffer
        self.effect.data.out_depth = target.depth_buf().map(|db| (db.as_output.clone(), (0, 0)));
    }
}
