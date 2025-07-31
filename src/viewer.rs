use anyhow::Result;
use image::{DynamicImage, GenericImageView};
use log::{debug, error, info};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
    keyboard::{KeyCode, PhysicalKey},
    dpi::LogicalSize,
};
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration,
    util::DeviceExt,
};
use crate::config::Config;
use crate::image_handler::ImageHandler;

/// テクスチャ情報
struct TextureInfo {
    bind_group: wgpu::BindGroup,
}

/// 画像ビューアー
pub struct ImageViewer {
    /// ウィンドウ
    window: Option<Arc<Window>>,
    /// WGPU インスタンス
    instance: Instance,
    /// WGPU サーフェス
    surface: Option<Surface<'static>>,
    /// WGPU アダプター
    adapter: Option<Adapter>,
    /// WGPU デバイス
    device: Option<Device>,
    /// WGPU キュー
    queue: Option<Queue>,
    /// サーフェス設定
    config: Option<SurfaceConfiguration>,
    /// レンダーパイプライン
    render_pipeline: Option<wgpu::RenderPipeline>,
    /// サンプラー
    sampler: Option<wgpu::Sampler>,
    /// バインドグループレイアウト
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    /// 現在のテクスチャ
    current_texture: Option<TextureInfo>,
    /// 頂点バッファ
    vertex_buffer: Option<wgpu::Buffer>,
    /// インデックスバッファ
    index_buffer: Option<wgpu::Buffer>,
    /// 設定
    app_config: Config,
    /// 画像ハンドラー
    image_handler: ImageHandler,
}

/// 頂点データ
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    /// 頂点属性を取得する
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 0.0] },
];

const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

impl ImageViewer {
    /// 新しい ImageViewer インスタンスを作成する
    /// 
    /// # Arguments
    /// * `config` - アプリケーション設定
    /// * `image_handler` - 画像ハンドラー
    /// 
    /// # Returns
    /// * `ImageViewer` - 画像ビューアー
    pub fn new(config: Config, image_handler: ImageHandler) -> Self {
        info!("画像ビューアーを初期化中...");

        // WGPU インスタンスを作成
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        ImageViewer {
            window: None,
            instance,
            surface: None,
            adapter: None,
            device: None,
            queue: None,
            config: None,
            render_pipeline: None,
            sampler: None,
            bind_group_layout: None,
            current_texture: None,
            vertex_buffer: None,
            index_buffer: None,
            app_config: config,
            image_handler,
        }
    }

    /// WGPU の初期化を行う
    async fn init_wgpu(&mut self) -> Result<()> {
        let window = self.window.as_ref().unwrap();
        
        // サーフェスを作成
        let surface = self.instance.create_surface(window.clone())?;

        // アダプターを取得
        let adapter = self.instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        // デバイスとキューを取得
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    label: None,
                    trace: wgpu::Trace::Off,
                }
            )
            .await?;

        // サーフェス設定
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.app_config.min_window_size.0,
            height: self.app_config.min_window_size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // シェーダーを作成
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/image.wgsl").into()),
        });

        // バインドグループレイアウトを作成
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        // レンダーパイプラインを作成
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // サンプラーを作成
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // 頂点バッファを作成
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // インデックスバッファを作成
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.surface = Some(surface);
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(surface_config);
        self.render_pipeline = Some(render_pipeline);
        self.sampler = Some(sampler);
        self.bind_group_layout = Some(bind_group_layout);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);

        info!("WGPUの初期化完了");
        Ok(())
    }

    /// 画像を読み込んでテクスチャを作成する
    /// 
    /// # Arguments
    /// * `image` - 読み込む画像
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn load_texture(&mut self, image: DynamicImage) -> Result<()> {
        debug!("テクスチャを作成中...");

        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let bind_group_layout = self.bind_group_layout.as_ref().unwrap();
        let sampler = self.sampler.as_ref().unwrap();

        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("image_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        self.current_texture = Some(TextureInfo {
            bind_group,
        });

        // ウィンドウサイズを調整
        self.adjust_window_size(dimensions.0, dimensions.1)?;

        debug!("テクスチャの作成完了");
        Ok(())
    }

    /// ウィンドウサイズを画像に合わせて調整する
    /// 
    /// # Arguments
    /// * `image_width` - 画像の幅
    /// * `image_height` - 画像の高さ
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    fn adjust_window_size(&mut self, image_width: u32, image_height: u32) -> Result<()> {
        let min_width = self.app_config.min_window_size.0;
        let min_height = self.app_config.min_window_size.1;

        // デスクトップの解像度を取得
        let desktop_size = self.get_desktop_size();
        let max_width = (desktop_size.0 as f32 * 0.9) as u32;
        let max_height = (desktop_size.1 as f32 * 0.9) as u32;

        let (new_width, new_height) = if image_width < min_width || image_height < min_height {
            // 最小サイズに収まる場合は拡大
            let scale_x = min_width as f32 / image_width as f32;
            let scale_y = min_height as f32 / image_height as f32;
            let scale = scale_x.min(scale_y);
            
            ((image_width as f32 * scale) as u32, (image_height as f32 * scale) as u32)
        } else if image_width > max_width || image_height > max_height {
            // デスクトップサイズを超える場合は縮小
            let scale_x = max_width as f32 / image_width as f32;
            let scale_y = max_height as f32 / image_height as f32;
            let scale = scale_x.min(scale_y);
            
            ((image_width as f32 * scale) as u32, (image_height as f32 * scale) as u32)
        } else {
            // そのままのサイズで表示
            (image_width, image_height)
        };

        if let Some(window) = &self.window {
            let _ = window.request_inner_size(LogicalSize::new(new_width, new_height));
        }
        self.resize(new_width, new_height);

        debug!("ウィンドウサイズを調整: {}x{}", new_width, new_height);
        Ok(())
    }

    /// デスクトップサイズを取得する
    /// 
    /// # Returns
    /// * `(u32, u32)` - デスクトップの幅と高さ
    #[cfg(windows)]
    fn get_desktop_size(&self) -> (u32, u32) {
        use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            (width, height)
        }
    }

    #[cfg(not(windows))]
    fn get_desktop_size(&self) -> (u32, u32) {
        (1920, 1080) // デフォルト値
    }

    /// ウィンドウのリサイズ処理
    /// 
    /// # Arguments
    /// * `new_width` - 新しい幅
    /// * `new_height` - 新しい高さ
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            if let (Some(surface), Some(device), Some(config)) = 
                (&self.surface, &self.device, &mut self.config) {
                config.width = new_width;
                config.height = new_height;
                surface.configure(device, config);
                debug!("ウィンドウをリサイズ: {}x{}", new_width, new_height);
            }
        }
    }

    /// 画面を描画する
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn render(&mut self) -> Result<()> {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let render_pipeline = self.render_pipeline.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let index_buffer = self.index_buffer.as_ref().unwrap();

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(render_pipeline);
            
            if let Some(texture_info) = &self.current_texture {
                render_pass.set_bind_group(0, &texture_info.bind_group, &[]);
            }
            
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// 現在の画像を読み込む
    fn load_current_image(&mut self) {
        if let Some(image_file) = self.image_handler.current_image() {
            let file_path = image_file.path.clone();
            let file_name = image_file.name.clone();
            info!("画像を読み込み中: {:?}", file_path);
            match self.image_handler.load_image(&file_path) {
                Ok(image) => {
                    if let Err(e) = self.load_texture(image) {
                        error!("テクスチャの読み込みに失敗: {:?}", e);
                    } else {
                        // ウィンドウタイトルを更新
                        self.update_window_title(&file_name);
                    }
                }
                Err(e) => {
                    error!("画像ファイルの読み込みに失敗: {:?}", e);
                }
            }
        }
    }

    /// ウィンドウタイトルを更新する
    /// 
    /// # Arguments
    /// * `filename` - 表示するファイル名
    fn update_window_title(&self, filename: &str) {
        if let Some(window) = &self.window {
            let title = format!("{} - rs_fast_image_viewer", filename);
            window.set_title(&title);
            debug!("ウィンドウタイトルを更新: {}", title);
        }
    }

    /// イベントループを実行する
    /// 
    /// # Arguments
    /// * `event_loop` - イベントループ
    /// 
    /// # Returns
    /// * `Result<()>` - 成功時は Ok(())
    pub fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {
        event_loop.run_app(&mut self)?;
        Ok(())
    }
}

impl ApplicationHandler for ImageViewer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("rs_fast_image_viewer")
                .with_inner_size(LogicalSize::new(
                    self.app_config.min_window_size.0,
                    self.app_config.min_window_size.1,
                ));
            
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window);

            // WGPUを初期化（非同期処理をブロック）
            pollster::block_on(async {
                if let Err(e) = self.init_wgpu().await {
                    error!("WGPUの初期化に失敗: {:?}", e);
                    event_loop.exit();
                    return;
                }

                // 最初の画像を読み込む
                self.load_current_image();
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if Some(window_id) != self.window.as_ref().map(|w| w.id()) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("アプリケーションを終了します");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) | PhysicalKey::Code(KeyCode::Enter) => {
                            info!("アプリケーションを終了します");
                            event_loop.exit();
                        }
                        PhysicalKey::Code(KeyCode::ArrowRight) | PhysicalKey::Code(KeyCode::KeyX) => {
                            self.image_handler.next_image();
                            self.load_current_image();
                        }
                        PhysicalKey::Code(KeyCode::ArrowLeft) | PhysicalKey::Code(KeyCode::KeyZ) => {
                            self.image_handler.previous_image();
                            self.load_current_image();
                        }
                        PhysicalKey::Code(KeyCode::F4) => {
                            // Alt+F4 の処理は OS レベルで処理される
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::RedrawRequested => {
                match self.render() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("描画エラー: {:?}", e);
                        if let Some(wgpu_err) = e.downcast_ref::<wgpu::SurfaceError>() {
                            match wgpu_err {
                                wgpu::SurfaceError::Lost => {
                                    if let Some(config) = &self.config {
                                        self.resize(config.width, config.height);
                                    }
                                }
                                wgpu::SurfaceError::OutOfMemory => event_loop.exit(),
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}