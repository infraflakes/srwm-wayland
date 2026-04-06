use smithay::{
    backend::renderer::{
        element::{Element, Id, Kind, RenderElement, UnderlyingStorage},
        gles::{
            GlesError, GlesFrame, GlesRenderer, GlesTexProgram, GlesTexture, Uniform,
            element::PixelShaderElement,
        },
        utils::{CommitCounter, OpaqueRegions},
    },
    output::Output,
    utils::{Logical, Physical, Point, Rectangle, Scale, Size, Transform},
};

use smithay::backend::allocator::Fourcc;

use super::BG_UNIFORMS;

const TILE_BG_SRC: &str = include_str!("../shaders/tile_bg.glsl");

pub const TILE_BG_UNIFORMS: &[smithay::backend::renderer::gles::UniformName<'static>] = &[
    smithay::backend::renderer::gles::UniformName {
        name: std::borrow::Cow::Borrowed("u_camera"),
        type_: smithay::backend::renderer::gles::UniformType::_2f,
    },
    smithay::backend::renderer::gles::UniformName {
        name: std::borrow::Cow::Borrowed("u_tile_size"),
        type_: smithay::backend::renderer::gles::UniformType::_2f,
    },
    smithay::backend::renderer::gles::UniformName {
        name: std::borrow::Cow::Borrowed("u_output_size"),
        type_: smithay::backend::renderer::gles::UniformType::_2f,
    },
];

pub fn compile_tile_bg_shader(renderer: &mut GlesRenderer) -> Option<GlesTexProgram> {
    match renderer.compile_custom_texture_shader(TILE_BG_SRC, TILE_BG_UNIFORMS) {
        Ok(shader) => Some(shader),
        Err(e) => {
            tracing::error!("Failed to compile tile background shader: {e}");
            None
        }
    }
}

/// Render element that tiles a texture across an area using a custom GLSL shader.
/// Behaves like `PixelShaderElement` for element tracking (stable ID, area-based
/// geometry, resize/update_uniforms) but renders via `render_texture_from_to`
/// so the shader can sample the tile texture.
#[derive(Debug, Clone)]
pub struct TileShaderElement {
    pub(super) shader: GlesTexProgram,
    pub(super) texture: GlesTexture,
    pub(super) tex_w: i32,
    pub(super) tex_h: i32,
    pub(super) id: Id,
    pub(super) commit_counter: CommitCounter,
    pub(super) area: Rectangle<i32, Logical>,
    pub(super) opaque_regions: Vec<Rectangle<i32, Logical>>,
    pub(super) alpha: f32,
    pub(super) additional_uniforms: Vec<Uniform<'static>>,
    pub(super) kind: Kind,
}

impl TileShaderElement {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shader: GlesTexProgram,
        texture: GlesTexture,
        tex_w: i32,
        tex_h: i32,
        area: Rectangle<i32, Logical>,
        opaque_regions: Option<Vec<Rectangle<i32, Logical>>>,
        alpha: f32,
        additional_uniforms: Vec<Uniform<'_>>,
        kind: Kind,
    ) -> Self {
        Self {
            shader,
            texture,
            tex_w,
            tex_h,
            id: Id::new(),
            commit_counter: CommitCounter::default(),
            area,
            opaque_regions: opaque_regions.unwrap_or_default(),
            alpha,
            additional_uniforms: additional_uniforms
                .into_iter()
                .map(|u| u.into_owned())
                .collect(),
            kind,
        }
    }

    pub fn resize(
        &mut self,
        area: Rectangle<i32, Logical>,
        opaque_regions: Option<Vec<Rectangle<i32, Logical>>>,
    ) {
        let opaque_regions = opaque_regions.unwrap_or_default();
        if self.area != area || self.opaque_regions != opaque_regions {
            self.area = area;
            self.opaque_regions = opaque_regions;
            self.commit_counter.increment();
        }
    }

    pub fn update_uniforms(&mut self, additional_uniforms: Vec<Uniform<'_>>) {
        self.additional_uniforms = additional_uniforms
            .into_iter()
            .map(|u| u.into_owned())
            .collect();
        self.commit_counter.increment();
    }
}

impl Element for TileShaderElement {
    fn id(&self) -> &Id {
        &self.id
    }
    fn current_commit(&self) -> CommitCounter {
        self.commit_counter
    }

    fn src(&self) -> Rectangle<f64, smithay::utils::Buffer> {
        Rectangle::from_size((self.tex_w as f64, self.tex_h as f64).into())
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.area.to_physical_precise_round(scale)
    }

    fn opaque_regions(&self, scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        self.opaque_regions
            .iter()
            .map(|region| region.to_physical_precise_round(scale))
            .collect()
    }

    fn alpha(&self) -> f32 {
        self.alpha
    }
    fn kind(&self) -> Kind {
        self.kind
    }
}

impl RenderElement<GlesRenderer> for TileShaderElement {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, smithay::utils::Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
        _cache: Option<&smithay::utils::user_data::UserDataMap>,
    ) -> Result<(), GlesError> {
        frame.render_texture_from_to(
            &self.texture,
            src,
            dst,
            damage,
            opaque_regions,
            Transform::Normal,
            self.alpha,
            Some(&self.shader),
            &self.additional_uniforms,
        )
    }

    #[inline]
    fn underlying_storage(&self, _renderer: &mut GlesRenderer) -> Option<UnderlyingStorage<'_>> {
        None
    }
}

/// Update the cached background shader element for the current camera/zoom.
/// Returns (camera_moved, zoom_changed) for the caller's damage logic.
pub fn update_background_element(
    state: &mut crate::state::Srwc,
    output: &Output,
    cur_camera: Point<f64, smithay::utils::Logical>,
    cur_zoom: f64,
    last_rendered_camera: Point<f64, smithay::utils::Logical>,
    last_rendered_zoom: f64,
) -> (bool, bool) {
    let camera_moved = cur_camera != last_rendered_camera;
    let zoom_changed = cur_zoom != last_rendered_zoom;
    let output_name = output.name();
    let output_size = crate::state::output_logical_size(output);
    let canvas_w = (output_size.w as f64 / cur_zoom).ceil() as i32;
    let canvas_h = (output_size.h as f64 / cur_zoom).ceil() as i32;
    let canvas_area = Rectangle::from_size((canvas_w, canvas_h).into());

    if let Some(elem) = state.render.cached_bg_elements.get_mut(&output_name) {
        elem.resize(canvas_area, Some(vec![canvas_area]));
        elem.update_uniforms(vec![Uniform::new(
            "u_camera",
            (cur_camera.x as f32, cur_camera.y as f32),
        )]);
    } else if let Some(elem) = state.render.cached_tile_bg.get_mut(&output_name) {
        elem.resize(canvas_area, Some(vec![canvas_area]));
        elem.update_uniforms(vec![
            Uniform::new("u_camera", (cur_camera.x as f32, cur_camera.y as f32)),
            Uniform::new("u_tile_size", (elem.tex_w as f32, elem.tex_h as f32)),
            Uniform::new("u_output_size", (canvas_w as f32, canvas_h as f32)),
        ]);
    }
    (camera_moved, zoom_changed)
}

/// Compile background shader and/or load tile image.
/// Called at startup and on config reload (lazy re-init).
/// On failure, falls back to `DEFAULT_SHADER` — never leaves background uninitialized.
pub fn init_background(
    state: &mut crate::state::Srwc,
    renderer: &mut GlesRenderer,
    initial_size: Size<i32, smithay::utils::Logical>,
    output_name: &str,
) {
    // Try loading tile image first (if configured and no shader_path)
    if state.config.background.shader_path.is_none()
        && let Some(path) = state.config.background.tile_path.as_deref()
    {
        match image::open(path) {
            Ok(img) => {
                let img = img.into_rgba8();
                let (w, h) = img.dimensions();
                let raw = img.into_raw();

                use smithay::backend::renderer::ImportMem;
                use smithay::utils::Buffer;
                match renderer.import_memory(
                    &raw,
                    Fourcc::Abgr8888,
                    Size::<i32, Buffer>::from((w as i32, h as i32)),
                    false,
                ) {
                    Ok(texture) => {
                        if state.render.tile_shader.is_none() {
                            state.render.tile_shader = compile_tile_bg_shader(renderer);
                        }
                        if let Some(ref shader) = state.render.tile_shader {
                            let tw = w as i32;
                            let th = h as i32;
                            let area = Rectangle::from_size(initial_size);
                            let elem = TileShaderElement::new(
                                shader.clone(),
                                texture,
                                tw,
                                th,
                                area,
                                Some(vec![area]),
                                1.0,
                                vec![
                                    Uniform::new("u_camera", (0.0f32, 0.0f32)),
                                    Uniform::new("u_tile_size", (tw as f32, th as f32)),
                                    Uniform::new(
                                        "u_output_size",
                                        (initial_size.w as f32, initial_size.h as f32),
                                    ),
                                ],
                                Kind::Unspecified,
                            );
                            state
                                .render
                                .cached_tile_bg
                                .insert(output_name.to_string(), elem);
                            return;
                        }
                        tracing::error!("Tile shader compilation failed, using default shader");
                    }
                    Err(e) => {
                        tracing::error!("Failed to upload tile texture: {e}, using default shader");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to load tile image {path}: {e}, using default shader");
            }
        }
    }

    // Reuse cached shader if already compiled (avoids redundant GPU work
    // when multiple outputs each need a background element).
    let shader = if let Some(ref cached) = state.render.background_shader {
        cached.clone()
    } else {
        let shader_source = if let Some(path) = state.config.background.shader_path.as_deref() {
            match std::fs::read_to_string(path) {
                Ok(src) => src,
                Err(e) => {
                    tracing::error!("Failed to read shader {path}: {e}, using default");
                    srwc::config::DEFAULT_SHADER.to_string()
                }
            }
        } else {
            srwc::config::DEFAULT_SHADER.to_string()
        };

        let compiled = match renderer.compile_custom_pixel_shader(&shader_source, BG_UNIFORMS) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to compile shader: {e}, using default");
                renderer
                    .compile_custom_pixel_shader(srwc::config::DEFAULT_SHADER, BG_UNIFORMS)
                    .expect("Default shader must compile")
            }
        };
        state.render.background_shader = Some(compiled.clone());
        compiled
    };

    let area = Rectangle::from_size(initial_size);
    state.render.cached_bg_elements.insert(
        output_name.to_string(),
        PixelShaderElement::new(
            shader,
            area,
            Some(vec![area]),
            1.0,
            vec![Uniform::new("u_camera", (0.0f32, 0.0f32))],
            Kind::Unspecified,
        ),
    );
}
