//! Operations (previously called "nodes")

use crate::param::Parameter;
use std::collections::HashMap;



/// Lifetimes:
/// - 'gctx : "global context" => long-lived objects, never freed
/// - 'ectx : "editor context" => objects that live until there is a modification of the network (Ops)
/// - 'fctx : "frame context" => objects that live for the current frame only
///
/// Nodes live in 'gctx.
/// Nodes create Ops that live in 'ectx, and can refer to 'gctx.
///
/// 'ectx kills:
/// - Modifications of the network structure
/// - Changing the size of the render target
///
/// 'fctx kills:
/// - end of frame
///
/// Some params can be evaluated in 'ectx (inside Node->Op), but others should be evaluated in 'fctx (inside Op).
/// Some params cannot be frame-context dependent (they cannot change every frame).
/// - e.g. render targets, shaders
pub trait Node<'a>
{
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);

    fn parameters(&self) -> Box<dyn Iterator<Item=&Parameter>>;
    fn parameters_mut(&mut self) -> Box<dyn Iterator<Item=&mut Parameter>>;

}

/// Execution context for render pass networks.
///
/// Provides:
/// - The arena allocator for frame-bound resources
/// - A command buffer into which it's possible to push rendering commands.
///     - The command buffer is executed within a render pass.
pub struct RenderPassContext;

/// Trait implemented by render pass operations.
///
/// Through this trait, ops declare the render targets they need.
/// (the ones they will read, the ones they will write, and the others they don't care about)
/// The op is responsible for building the argblocks passed to the shader.
///
/// RenderPassNodes create RenderPassOps that live in 'ectx.
/// The RenderPassOps contain allocated resources needed by the render pass,
/// and references to the render targets they need (either 'ectx or 'fctx)
///
/// A render target is simply an image (autograph_api::Image<B>). A render pass node can query
/// the context for a reference to an image given an input parameter path.
/// Of course, this pushes onto the execution stack. Ultimately, will end up on the
/// node that creates the render targets (and allocates them into the 'ectx arena).
/// Same principle for other resources.
///
/// Within a renderpass, all render targets are grouped. Cannot sample within the same renderpass.
/// All ops must share the same render target layout.
/// How do we ensure that? => One single render target set per renderpass.
pub trait RenderNode: Node
{
    // bake(RenderPassBakeContext) -> Box<dyn RenderPassOp>
    // fn make_op(&mut self, &mut RenderContext) -> Box<dyn RenderOp>
    // geometry_format() -> GeometryRequirements
}

// GeometryRequirements is basically a bunch of semantics and vertex formats


pub trait RenderOp
{
    // execute
    // fn make_geometry_sink(ctx: &mut RenderExecContext) -> RenderGeometrySink
}

/// Created by the renderer at the beginning of every frame.
/// Can refer to the parent Op that created it.
pub trait RenderGeometrySink {
    // called by the scene filter to render geometry
    // fn draw(ctx: &mut RenderGeometryContext, geom: &GeometryBatch)
    // GeometryBatch is Vertexbuffers + draw call parameters (possibly more than one call)
    // build uniform buffers & upload, build per-draw argblocks
}


pub struct RenderExecContext {
    // cmdbuf
    // &SceneFilterContext
}

/// Render ops.
/// Can query images (or image groups) from Node context.
/// Can query temporary resources (temp buffers & images).
pub struct RenderContext {
    // query image
    // fn image(&self, path: &str) -> Result<&'a Image, RenderContextError>
    // fn image_view(), texture_image_view(), render_target_view() ...
    // fn transient_image(&self) -> ImageBuilder<Image<'a>,_>
    // fn buffer(&self, path: &str) -> Result<&'a Buffer, RenderContextError>
    // fn commandbuffer()
    //      -> API: create_command_buffer()
    // fn create_typed_pipeline()
    //      -> API: create_typed_pipeline()

    // ctx.image(path)?.subimage(subimage_path)?
    // image() can also load images from file (using file: URLs)
    // -> file://
    // -> http://
    // -> node:/render/subnet/0
    // -> node:local
    // -> and other registerable protocol handlers
    //      -> trait ImageProtocolHandler (loads image data to GPU, or simply returns pointer to data, and image metadata)
}

// Three 'layers' of objects corresponding to different lifetimes
// RenderNode (>'ectx) -> RenderOp ('ectx) -> RenderGeometrySink ('fctx)
//
// RenderNode
// - pipelines
// - render targets
// RenderOp
// - per-frame targets
// - per-frame uniforms
// RenderGeometrySink
// - per-object uniforms

// Error handling:
// - ContextError
//      - RenderContextError
//          - (predefined error conditions)
//          - Other(source: Box<dyn Error>)
//          - From(Error) -> Other
//      - RenderFrameContextError
//      -


// Validation of parameters
// -> Result<(),ValidationError>
// - ValidationContext (when all inputs are specified)
//      - RenderValidationContext
//          - validate.image(path).require_usage(...).require_size(...).require_layers(...)
// - on re-validation: remove all constraints on all inputs from all contexts
//      - context.unplug(from, to)

// User interface:
// - display widgets for parameters
// - display reflected data (e.g. inferred metadata on inputs/outputs)

// Issue: know usage of resources before allocation
// -> with parameters, pass requirement metadata
// -> with params, pass validation callback
// - trait Parameter
//      - ImageParameter: Parameter
//           - requirements: Vec<ImageRequest> (soft requirements & hard requirements)
//                  - ImageRequest::Size
//                  - ImageRequest::Usage