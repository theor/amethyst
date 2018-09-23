//! A stage in the rendering pipeline.

use amethyst_core::specs::prelude::SystemData;
use amethyst_assets::{Loader, AssetStorage};
use error::{Error, Result};
use fnv::FnvHashMap as HashMap;
use hetseq::*;
use pipe::pass::{CompiledPass, Pass, PassData};
use pipe::{Target, Targets, Program};
use types::{Encoder, Factory};
use super::super::Renderer;

/// A stage in the rendering pipeline.
#[derive(Clone, Debug)]
pub struct Stage<L> {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    passes: L,
    target_name: String,
    target: Target,
}

impl Stage<List<()>> {
    /// Builds a new `PolyStage` which outputs to the `Target` with the given name.
    pub fn with_target<N: Into<String>>(target_name: N) -> StageBuilder<Queue<()>> {
        StageBuilder::new(target_name.into())
    }

    /// Builds a new `PolyStage` which outputs straight into the backbuffer.
    pub fn with_backbuffer() -> StageBuilder<Queue<()>> {
        StageBuilder::new("")
    }
}

impl<L> Stage<L> {
    /// Enables the `PolyStage` so it will execute on every frame.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the `PolyStage`, preventing it from being executed on every frame.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Returns whether this `PolyStage` is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

pub trait PassesData<'a> {
    type Data: SystemData<'a> + Send;
}

pub trait Passes: for<'a> PassesData<'a> {
    fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
        data: <Self as PassesData<'b>>::Data,
    );
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <Self as PassesData<'b>>::Data,
    );

    /// Distributes new targets
    fn new_target(&mut self, new_target: &Target);
}

impl<'a, HP> PassesData<'a> for List<(CompiledPass<HP>, List<()>)>
where
    HP: Pass,
{
    type Data = <HP as PassData<'a>>::Data;
}

impl<HP> Passes for List<(CompiledPass<HP>, List<()>)>
where
    HP: Pass,
{
    fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
        hd: <HP as PassData<'b>>::Data,
    ) {
        let List((ref mut hp, _)) = *self;
        hp.reload(renderer, storage, hd);
    }
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        hd: <HP as PassData<'b>>::Data,
    ) {
        let List((ref mut hp, _)) = *self;
        hp.apply(encoder, factory, hd);
    }

    fn new_target(&mut self, new_target: &Target) {
        let List((ref mut hp, _)) = *self;
        hp.new_target(new_target);
    }
}

impl<'a, HP, TP> PassesData<'a> for List<(CompiledPass<HP>, TP)>
where
    HP: Pass,
    TP: Passes,
{
    type Data = (<HP as PassData<'a>>::Data, <TP as PassesData<'a>>::Data);
}

impl<HP, TP> Passes for List<(CompiledPass<HP>, TP)>
where
    HP: Pass,
    TP: Passes,
{
    fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
        (hd, td): <Self as PassesData<'b>>::Data,
    ) {
        let List((ref mut hp, ref mut tp)) = *self;
        hp.reload(renderer, storage, hd);
        tp.reload(renderer, storage, td);
    }
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        (hd, td): (<HP as PassData<'b>>::Data, <TP as PassesData<'b>>::Data),
    ) {
        let List((ref mut hp, ref mut tp)) = *self;
        hp.apply(encoder, factory.clone(), hd);
        tp.apply(encoder, factory, td);
    }

    fn new_target(&mut self, new_target: &Target) {
        let List((ref mut hp, ref mut tp)) = *self;
        hp.new_target(new_target);
        tp.new_target(new_target);
    }
}

/// Data requested by the pass from the specs::World.
pub trait StageData<'a> {
    type Data: SystemData<'a> + Send;
}

/// A stage in the rendering.  Contains multiple passes.
pub trait PolyStage: for<'a> StageData<'a> {
    ///
    fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
        data: <Self as StageData<'b>>::Data,
    );
    ///
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <Self as StageData<'b>>::Data,
    );

    /// Distributes new targets
    fn new_targets(&mut self, new_targets: &HashMap<String, Target>);
}

impl<'a, L> StageData<'a> for Stage<L>
where
    L: Passes,
{
    type Data = <L as PassesData<'a>>::Data;
}

impl<L> PolyStage for Stage<L>
where
    L: Passes + Length,
{
    fn reload<'a, 'b: 'a>(
        &'a mut self,
        renderer: &'a mut Renderer,
        storage: &'a AssetStorage<Program>,
        data: <Self as StageData<'b>>::Data,
    ) {
        self.passes.reload(renderer, storage, data);
    }
    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        factory: Factory,
        data: <L as PassesData<'b>>::Data,
    ) {
        self.clear_color
            .map(|c| self.target.clear_color(encoder, c));
        self.clear_depth
            .map(|d| self.target.clear_depth_stencil(encoder, d));

        self.passes.apply(encoder, factory, data);
    }

    fn new_targets(&mut self, new_targets: &HashMap<String, Target>) {
        match new_targets.get(&self.target_name) {
            Some(target) => {
                self.target = target.clone();
                self.passes.new_target(target);
            }
            None => {
                error!("Target name {:?} not found!", self.target_name);
            }
        }
    }
}

/// Constructs a new rendering stage.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub struct StageBuilder<Q> {
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    enabled: bool,
    passes: Q,
    target_name: String,
}

impl StageBuilder<Queue<()>> {
    /// Creates a new `StageBuilder` using the given target.
    pub fn new<T: Into<String>>(target_name: T) -> Self {
        StageBuilder {
            clear_color: None,
            clear_depth: None,
            enabled: true,
            passes: Queue::new(),
            target_name: target_name.into(),
        }
    }
}

impl<Q> StageBuilder<Q> {
    /// Clears the stage's target.
    pub fn clear_target<R, C, D>(mut self, color_val: C, depth_val: D) -> Self
    where
        R: Into<[f32; 4]>,
        C: Into<Option<R>>,
        D: Into<Option<f32>>,
    {
        self.clear_color = color_val.into().map(|c| c.into());
        self.clear_depth = depth_val.into();
        self
    }

    /// Sets whether the `PolyStage` is turned on by default.
    pub fn enabled(mut self, val: bool) -> Self {
        self.enabled = val;
        self
    }

    pub(crate) fn build<'a, L, Z, R>(
        self,
        renderer: &'a mut Renderer,
        targets: &'a Targets,
        multisampling: u16,
        loader: &'a Loader,
        storage: &'a AssetStorage<Program>
    ) -> Result<Stage<R>>
    where
        Q: IntoList<List = L>,
        L: for<'b> Functor<CompilePass<'b>, Output = Z>,
        Z: Try<Error, Ok = R>,
        R: Passes,
    {
        let out = targets
            .get(&self.target_name)
            .cloned()
            .ok_or(Error::NoSuchTarget(self.target_name.clone()))?;

        let passes = self
            .passes
            .into_list()
            .fmap(CompilePass::new(renderer, &out, multisampling, loader, storage))
            .try()?;

        Ok(Stage {
            clear_color: self.clear_color,
            clear_depth: self.clear_depth,
            enabled: self.enabled,
            passes,
            target: out,
            target_name: self.target_name,
        })
    }
}

impl<'a, Q> StageBuilder<Queue<Q>> {
    /// Appends another `Pass` to the stage.
    pub fn with_pass<P: Pass>(self, pass: P) -> StageBuilder<Queue<(Queue<Q>, P)>> {
        StageBuilder {
            clear_color: self.clear_color,
            clear_depth: self.clear_depth,
            enabled: self.enabled,
            passes: self.passes.push(pass),
            target_name: self.target_name,
        }
    }
}

pub struct CompilePass<'a> {
    renderer: &'a mut Renderer,
    target: &'a Target,
    multisampling: u16,
    loader: &'a Loader,
    storage: &'a AssetStorage<Program>
}

impl<'a> CompilePass<'a> {
    fn new(renderer: &'a mut Renderer, target: &'a Target, multisampling: u16, loader: &'a Loader, storage: &'a AssetStorage<Program>
        ) -> Self {
        CompilePass {
            renderer,
            target,
            multisampling,
            loader,
            storage,
        }
    }
}

impl<'a, P> HetFnOnce<(P,)> for CompilePass<'a>
where
    P: Pass,
{
    type Output = Result<CompiledPass<P>>;
    fn call_once(self, (pass,): (P,)) -> Result<CompiledPass<P>> {
        CompiledPass::compile(pass, self.renderer, self.target, self.multisampling, self.loader, self.storage)
    }
}
impl<'a, P> HetFnMut<(P,)> for CompilePass<'a>
where
    P: Pass,
{
    fn call_mut(&mut self, (pass,): (P,)) -> Result<CompiledPass<P>> {
        CompiledPass::compile(pass, self.renderer, self.target, self.multisampling, self.loader, self.storage)
    }
}
