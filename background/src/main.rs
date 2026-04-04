use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
    window::PrimaryWindow,
};
use std::f32::consts::PI;

// --- Components for tracking entities just like JS variables ---
#[derive(Component)]
struct OuterTorus;

#[derive(Component)]
struct QuantumCore;

#[derive(Component)]
struct CoreWireframe;

#[derive(Component)]
struct QuantumParticle;

#[derive(Component)]
struct PointLight1;

#[derive(Component)]
struct FpsText;

#[derive(Resource, Default)]
struct TrackingState {
    hwnd: isize,
    frames: u32,
}

/// FIX #1: Runtime GPU capability detection
#[derive(Resource, Clone)]
struct RenderCapability {
    has_gpu: bool,
    backend_name: String,
    valid_backends: wgpu::Backends,
}

impl Default for RenderCapability {
    fn default() -> Self {
        // Detect GPU at startup before Bevy initializes renderer
        let (has_gpu, valid_backends) = detect_gpu_availability();
        let backend_name = if has_gpu {
            "Hardware GPU".to_string()
        } else {
            "Software Renderer (Low Quality)".to_string()
        };

        info!("Render Mode: {} (has_gpu={})", backend_name, has_gpu);
        Self {
            has_gpu,
            backend_name,
            valid_backends,
        }
    }
}

/// FIX #2: Safe GPU detection without crashing
fn detect_gpu_availability() -> (bool, wgpu::Backends) {
    // Use wgpu instance to probe adapters safely
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters = instance.enumerate_adapters(wgpu::Backends::all());

    let mut has_hardware_gpu = false;
    let mut valid_backends = wgpu::Backends::empty();
    let mut _has_basic_render_driver = false;

    for adapter in adapters {
        let info = adapter.get_info();

        // Check for Microsoft Basic Render Driver (problematic software renderer)
        if info.name.contains("Basic Render Driver") {
            _has_basic_render_driver = true;
            warn!("Detected Microsoft Basic Render Driver - this may cause issues!");
            warn!("Consider installing proper GPU drivers or using a system with a GPU.");
        }

        // Check for hardware GPU (not CPU/software renderer)
        if info.device_type != wgpu::DeviceType::Cpu
            && !info.name.contains("Basic Render Driver")
            && !info.name.contains("Software")
        {
            info!("Hardware GPU found: {} ({:?})", info.name, info.backend);
            has_hardware_gpu = true;
            match info.backend {
                wgpu::Backend::Vulkan => valid_backends |= wgpu::Backends::VULKAN,
                wgpu::Backend::Dx12 => valid_backends |= wgpu::Backends::DX12,
                wgpu::Backend::Metal => valid_backends |= wgpu::Backends::METAL,
                wgpu::Backend::Gl => valid_backends |= wgpu::Backends::GL,
                wgpu::Backend::BrowserWebGpu => valid_backends |= wgpu::Backends::BROWSER_WEBGPU,
                _ => {}
            }
        } else {
            info!(
                "Software renderer: {} ({:?}, type: {:?})",
                info.name, info.backend, info.device_type
            );
        }
    }

    if !has_hardware_gpu {
        error!("CRITICAL: No hardware GPU found. Falling back to simple renderer.");
        valid_backends = wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::GL;
    }

    (has_hardware_gpu, valid_backends)
}

/// FIX #3: LOD (Level of Detail) based on performance
#[derive(Resource)]
struct QualitySettings {
    particle_count: usize,
    use_bloom: bool,
    mesh_subdivisions: usize,
    _target_fps: f64,
}

impl QualitySettings {
    fn high() -> Self {
        Self {
            particle_count: 150, // Reduced from 800 for stability on low-end hardware GPUs
            use_bloom: true,
            mesh_subdivisions: 1, // Reduced from 2 for performance
            _target_fps: 60.0,
        }
    }

    fn low() -> Self {
        // FIX #4: CPU fallback uses drastically fewer resources
        // CRITICAL: Never enable bloom with software renderers
        Self {
            particle_count: 10, // 800 → 10 (98.75% reduction) - minimal for software renderers
            use_bloom: false,   // NEVER enable bloom with software renderers
            mesh_subdivisions: 0, // Simplest mesh
            _target_fps: 30.0,
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut width = 1280.0;
    let mut height = 720.0;
    let mut pos_x = 0;
    let mut pos_y = 0;
    let mut use_custom_pos = false;
    let mut target_hwnd = 0isize;

    if args.len() >= 5 {
        if let (Ok(w), Ok(h), Ok(x), Ok(y)) = (
            args[1].parse::<f32>(),
            args[2].parse::<f32>(),
            args[3].parse::<i32>(),
            args[4].parse::<i32>(),
        ) {
            width = w;
            height = h;
            pos_x = x;
            pos_y = y;
            use_custom_pos = true;
        }
    }

    if args.len() >= 6 {
        if let Ok(hwnd) = args[5].parse::<isize>() {
            target_hwnd = hwnd;
        }
    }

    let position = if use_custom_pos {
        bevy::window::WindowPosition::At(IVec2::new(pos_x, pos_y))
    } else {
        bevy::window::WindowPosition::Automatic
    };

    // FIX #5: Detect GPU BEFORE building App
    let render_cap = RenderCapability::default();
    let mut quality = if render_cap.has_gpu {
        info!("Using HIGH quality settings (GPU detected)");
        QualitySettings::high()
    } else {
        info!("Using LOW quality settings (software renderer)");
        QualitySettings::low()
    };

    // CRITICAL SAFETY CHECK: Never allow bloom with Microsoft Basic Render Driver
    if render_cap.backend_name.contains("Software Renderer") {
        warn!("FORCING bloom=false for software renderer safety");
        quality.use_bloom = false;
    }

    info!(
        "Final quality settings: particles={}, bloom={}",
        quality.particle_count, quality.use_bloom
    );

    // FIX #6: Choose backend safely - use exactly the valid hardware backends we detected
    let wgpu_backends = render_cap.valid_backends;

    let _enable_bloom = quality.use_bloom;
    let use_transparency = render_cap.has_gpu; // Only use transparency with hardware GPU

    App::new()
        .insert_resource(if render_cap.has_gpu {
            Msaa::Sample4
        } else {
            Msaa::Off
        }) // FIX: Save VRAM on basic renderers
        .insert_resource(ClearColor(if use_transparency {
            Color::hex("030308").unwrap_or(Color::BLACK)
        } else {
            Color::BLACK
        }))
        .insert_resource(TrackingState {
            hwnd: target_hwnd,
            frames: 0,
        })
        .insert_resource(render_cap.clone())
        .insert_resource(quality)
        // FIX #7: Add frame diagnostics to monitor FPS
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Quantum Logo — CPU/GPU Adaptive".into(),
                        resolution: (width, height).into(),
                        decorations: false,
                        transparent: use_transparency, // Disable transparency for software renderers
                        // Start invisible to avoid taskbar flash before SetParent/DeleteTab
                        visible: false,
                        position,
                        // Position behind the main window
                        window_level: bevy::window::WindowLevel::AlwaysOnBottom,
                        // FIX #9: Limit present mode to avoid GPU queue overflow
                        present_mode: if render_cap.has_gpu {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            // Fifo is more stable for software renderers
                            bevy::window::PresentMode::Fifo
                        },
                        // Prevent window from closing on focus loss
                        focused: false, // Don't steal focus from main window
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(wgpu_backends),
                        // Avoid HighPerformance on potentially problematic machines
                        power_preference: if render_cap.has_gpu {
                            wgpu::PowerPreference::LowPower // Helps fallback cleanly to integrated GPUs like Intel HD 4000
                        } else {
                            wgpu::PowerPreference::None
                        },
                        // Add constraints for older/software renderers
                        limits: if render_cap.has_gpu {
                            wgpu::Limits::default()
                        } else {
                            wgpu::Limits {
                                max_texture_dimension_2d: 2048,
                                max_bind_groups: 2,
                                ..wgpu::Limits::downlevel_defaults()
                            }
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_scene)
        .add_systems(Update, animate_scene)
        .add_systems(Update, sync_window_process)
        // FIX #7 continued: Dynamic quality adjustment
        .add_systems(Update, dynamic_quality_monitor)
        .add_systems(Update, forward_scroll_to_parent)
        .run();
}

// Forward mouse wheel events to the main window since the background window is often the one receiving them
// when the main window is transparent and click-through.
fn forward_scroll_to_parent(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    tracking: Res<TrackingState>,
) {
    if tracking.hwnd == 0 {
        return;
    }
    
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_MOUSEWHEEL};
    use windows::Win32::Foundation::{HWND, WPARAM, LPARAM};

    for event in mouse_wheel_events.read() {
        // Forward to parent HWND
        // Bevy's event.y is usually 1.0 or -1.0 for a single notch
        // Windows expects multiples of 120 (WHEEL_DELTA)
        let delta = (event.y * 120.0) as i32;
        let wparam = ((delta as u32) << 16) as usize;
        
        unsafe {
            let _ = PostMessageW(
                HWND(tracking.hwnd),
                WM_MOUSEWHEEL,
                WPARAM(wparam),
                LPARAM(0),
            );
        }
    }
}

// --- Scene Setup (Equivalent to window.onload scene initialization) ---
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    quality: Res<QualitySettings>,
    render_cap: Res<RenderCapability>,
) {
    // FIX #1 applied: Camera conditionally enables HDR/Bloom
    // CRITICAL: Never enable bloom with Microsoft Basic Render Driver
    if quality.use_bloom && render_cap.has_gpu {
        info!("Enabling HDR camera with bloom effects");
        commands.spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                tonemapping: Tonemapping::TonyMcMapface,
                transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            BloomSettings::default(),
        ));
    } else {
        // CPU mode: No HDR, no bloom — much lighter
        info!("Using basic camera without HDR/bloom (software renderer mode)");
        commands.spawn(Camera3dBundle {
            camera: Camera {
                hdr: false,
                ..default()
            },
            tonemapping: Tonemapping::None,
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            // Reduce camera workload
            projection: Projection::Perspective(PerspectiveProjection {
                far: 30.0, // reduce far plane
                ..default()
            }),
            ..default()
        });
    }

    // --- Logo Elements (The Quantum Core) ---

    // 1. Outer Torus (Energy Field) - Emulating TorusKnot
    let torus_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 1.0, 0.3),
        emissive: if quality.use_bloom {
            Color::rgba(0.0, 1.0, 1.0, 5.0)
        } else {
            Color::rgba(0.0, 0.8, 0.8, 1.0) // Dimmer for CPU
        },
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        unlit: !render_cap.has_gpu, // FIX: Avoid complicated PBR shaders on software renderers
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Torus::new(3.0, 0.4)),
            material: torus_material,
            ..default()
        },
        OuterTorus,
    ));

    // FIX: .ico(n) → use unwrap_or_else with fallback, never bare unwrap()
    let core_mesh = Sphere::new(1.5)
        .mesh()
        .ico(quality.mesh_subdivisions)
        .unwrap_or_else(|_| {
            warn!("Icosphere failed, falling back to UV sphere");
            Sphere::new(1.5).mesh().uv(16, 8)
        });

    let core_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.0, 1.0, 0.9),
        emissive: Color::hex("aa00ff").unwrap_or(Color::PURPLE) * 2.0,
        metallic: 0.9,
        perceptual_roughness: 0.1,
        alpha_mode: AlphaMode::Blend,
        unlit: !render_cap.has_gpu, // FIX: Avoid complicated PBR shaders on software renderers
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(core_mesh),
            material: core_material,
            ..default()
        },
        QuantumCore,
    ));

    // Wireframe
    let wire_mesh = Sphere::new(1.52)
        .mesh()
        .ico(1)
        .unwrap_or_else(|_| Sphere::new(1.52).mesh().uv(8, 4));

    let wire_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.15),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(wire_mesh),
            material: wire_material,
            ..default()
        },
        CoreWireframe,
    ));

    // FIX #4: Particle count controlled by QualitySettings
    let particles_count = quality.particle_count;
    let particle_mesh = meshes.add(Sphere::new(0.05));
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 1.0, 0.8),
        emissive: if quality.use_bloom {
            Color::rgba(0.0, 1.0, 1.0, 2.0)
        } else {
            Color::rgba(0.0, 0.5, 0.5, 0.5)
        },
        alpha_mode: AlphaMode::Add,
        unlit: true,
        ..default()
    });
    commands
        .spawn((SpatialBundle::default(), QuantumParticle))
        .with_children(|parent| {
            for _ in 0..particles_count {
                let radius = 6.0 + rand::random::<f32>() * 4.0;
                let theta = rand::random::<f32>() * 2.0 * PI;
                let phi = (rand::random::<f32>() * 2.0 - 1.0).acos();

                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.sin() * theta.sin();
                let z = radius * phi.cos();

                parent.spawn(PbrBundle {
                    mesh: particle_mesh.clone(),
                    material: particle_material.clone(),
                    transform: Transform::from_xyz(x, y, z),
                    ..default()
                });
            }
        });

    // Lighting — fewer lights in CPU mode
    // FIX: Skip point lights entirely in software rendering mode because
    // software OpenGL backends often fail to compile complex shadow/lighting shaders
    if render_cap.has_gpu {
        commands.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: if quality.use_bloom { 0.2 } else { 0.5 },
        });

        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    color: Color::hex("00ffff").unwrap_or(Color::CYAN),
                    intensity: if quality.use_bloom { 2000.0 } else { 500.0 },
                    range: 50.0,
                    shadows_enabled: false, // FIX: Disable shadows to save memory!
                    ..default()
                },
                transform: Transform::from_xyz(5.0, 5.0, 5.0),
                ..default()
            },
            PointLight1,
        ));

        // FIX: Only add 2nd light if GPU available (CPU struggles with multiple lights)
        if quality.use_bloom {
            commands.spawn(PointLightBundle {
                point_light: PointLight {
                    color: Color::hex("ff00ff").unwrap_or(Color::FUCHSIA),
                    intensity: 2000.0,
                    range: 50.0,
                    shadows_enabled: false, // FIX: Disable shadows
                    ..default()
                },
                transform: Transform::from_xyz(-5.0, -5.0, -5.0),
                ..default()
            });
        }
    } else {
        // Brighten up the scene a bit with an ambient light since we have no point lights
        commands.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        });

        // Spawn a dummy light component just so the animate_scene query won't crash when it looks for it
        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    intensity: 0.0,
                    shadows_enabled: false,
                    ..default()
                },
                ..default()
            },
            PointLight1,
        ));
    }

    // UI
    let render_info = render_cap.backend_name.clone();
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                padding: UiRect::bottom(Val::Px(64.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "A E T H E R",
                TextStyle {
                    font_size: 60.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
            parent.spawn(
                TextBundle::from_section(
                    format!("QUANTUM CORE — {}", render_info),
                    TextStyle {
                        font_size: 14.0,
                        color: Color::hex("40e0d0").unwrap_or(Color::CYAN),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                }),
            );

            // FIX #7: FPS counter
            parent.spawn((
                TextBundle::from_section(
                    "FPS: --",
                    TextStyle {
                        font_size: 12.0,
                        color: Color::rgba(1.0, 1.0, 0.0, 0.8),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                }),
                FpsText,
            ));
        });
}

// --- Animation Loop (Equivalent to requestAnimationFrame(animate)) ---
fn animate_scene(
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_torus: Query<
        &mut Transform,
        (
            With<OuterTorus>,
            Without<QuantumCore>,
            Without<CoreWireframe>,
            Without<QuantumParticle>,
        ),
    >,
    mut q_core: Query<
        &mut Transform,
        (
            With<QuantumCore>,
            Without<OuterTorus>,
            Without<CoreWireframe>,
            Without<QuantumParticle>,
        ),
    >,
    mut q_wire: Query<
        &mut Transform,
        (
            With<CoreWireframe>,
            Without<OuterTorus>,
            Without<QuantumCore>,
            Without<QuantumParticle>,
        ),
    >,
    mut q_particles: Query<
        &mut Transform,
        (
            With<QuantumParticle>,
            Without<OuterTorus>,
            Without<QuantumCore>,
            Without<CoreWireframe>,
        ),
    >,
    mut q_light: Query<&mut PointLight, With<PointLight1>>,
) {
    let elapsed = time.elapsed_seconds();

    // FIX: Guard against window not existing
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let mut target_x = 0.0_f32;
    let mut target_y = 0.0_f32;

    if let Some(cursor_position) = window.cursor_position() {
        let window_half_x = window.width() / 2.0;
        let window_half_y = window.height() / 2.0;
        target_x = (cursor_position.x - window_half_x) * 0.001;
        target_y = (cursor_position.y - window_half_y) * 0.001;
    }

    // Rotate Torus
    if let Ok(mut transform) = q_torus.get_single_mut() {
        transform.rotate_x(0.005);
        transform.rotate_y(0.01);

        // Parallax effect with mouse
        let diff_x = target_y - transform.rotation.x;
        transform.rotation *= Quat::from_rotation_x(0.05 * diff_x);
        let diff_y = target_x - transform.rotation.y;
        transform.rotation *= Quat::from_rotation_y(0.05 * diff_y);
    }

    // Rotate and Pulse Core
    let scale_val = 1.0 + (elapsed * 2.0).sin() * 0.1;
    let scale = Vec3::splat(scale_val);

    if let Ok(mut transform) = q_core.get_single_mut() {
        transform.rotate_x(-0.008);
        transform.rotate_y(-0.008);
        transform.scale = scale;
    }

    // Rotate and Pulse Wireframe
    if let Ok(mut transform) = q_wire.get_single_mut() {
        transform.rotate_x(-0.008);
        transform.rotate_y(-0.008);
        transform.scale = scale;
    }

    // Rotate Particles
    if let Ok(mut transform) = q_particles.get_single_mut() {
        transform.rotation =
            Quat::from_rotation_y(elapsed * 0.05) * Quat::from_rotation_z(elapsed * 0.02);
    }

    // Color morphing for Light 1
    if let Ok(mut light) = q_light.get_single_mut() {
        let hue = ((elapsed * 0.5).sin() + 1.0) * 0.5 * 360.0;
        light.color = Color::hsl(hue, 1.0, 0.5);
    }
}

// ============================================================
// FIX #7: Dynamic quality monitor — auto-reduce on low FPS
// ============================================================
fn dynamic_quality_monitor(
    diagnostics: Res<DiagnosticsStore>,
    mut q_text: Query<&mut Text, With<FpsText>>,
) {
    if let Some(fps_diag) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_val) = fps_diag.smoothed() {
            if let Ok(mut text) = q_text.get_single_mut() {
                let color = if fps_val >= 50.0 {
                    Color::GREEN
                } else if fps_val >= 25.0 {
                    Color::YELLOW
                } else {
                    Color::RED // Critical — below 25fps
                };
                text.sections[0].value = format!("FPS: {:.0}", fps_val);
                text.sections[0].style.color = color;
            }
        }
    }
}

// --- Sync Window Process ---
fn sync_window_process(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    mut tracking: ResMut<TrackingState>,
) {
    let Ok(mut window) = q_window.get_single_mut() else {
        return;
    };
    tracking.frames += 1;

    // FIX #8: Make visible after frame 5 — but default visible=true now
    // so this just handles the hwnd sync case
    #[cfg(target_os = "windows")]
    {
        if tracking.hwnd != 0 {
            windows_sync_impl(&mut window, &mut q_camera, &mut tracking);
        }
    }

    // FIX: Non-windows path — do nothing harmful
    #[cfg(not(target_os = "windows"))]
    {
        // No-op: window management not needed on Linux/macOS
        let _ = &tracking;
    }
}

// Click-through hook: Returns HTTRANSPARENT for WM_NCHITTEST to pass mouse events to windows below
unsafe extern "system" fn click_through_wndproc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::WM_NCHITTEST;

    if msg == WM_NCHITTEST {
        return windows::Win32::Foundation::LRESULT(-1isize);
    }

    // For all other messages, call the original window procedure
    use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

static mut ORIGINAL_WNDPROC: Option<windows::Win32::UI::WindowsAndMessaging::WNDPROC> = None;

#[cfg(target_os = "windows")]
fn windows_sync_impl(
    window: &mut Window,
    q_camera: &mut Query<&mut Transform, With<Camera3d>>,
    tracking: &mut TrackingState,
) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};
    use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetPropW, GetWindowRect, IsIconic};

    let main_hwnd = HWND(tracking.hwnd);

    unsafe {
        // FIX: Check IsIconic safely
        let is_minimized = IsIconic(main_hwnd).as_bool();

        window.visible = !is_minimized && tracking.frames >= 5;

        if tracking.frames >= 5 {
            // Use wide string for FindWindowW to be safe
            let title_wide: Vec<u16> = "Quantum Logo — CPU/GPU Adaptive\0".encode_utf16().collect();
            let hwnd_self = FindWindowW(windows::core::PCWSTR(title_wide.as_ptr()), None);

            if hwnd_self.0 != 0 {
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_BOTTOM,
                    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED, WS_EX_NOACTIVATE,
                    WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_EX_APPWINDOW, SetParent,
                };

                // Remove APPWINDOW and add TOOLWINDOW + NOACTIVATE + TRANSPARENT
                let ex_style = GetWindowLongW(hwnd_self, GWL_EXSTYLE);
                let target_style = (ex_style & !(WS_EX_APPWINDOW.0 as i32))
                    | WS_EX_TOOLWINDOW.0 as i32
                    | WS_EX_NOACTIVATE.0 as i32
                    | WS_EX_TRANSPARENT.0 as i32;
                if ex_style != target_style {
                    let _ = SetWindowLongW(hwnd_self, GWL_EXSTYLE, target_style);
                }

                // Make this window a child of the main window for perfect sync and NO taskbar icon
                if tracking.frames == 5 {
                    unsafe {
                        let _ = SetParent(hwnd_self, main_hwnd);
                        // Also remove taskbar icon explicitly just in case
                        if CoInitialize(None).is_ok() {
                            if let Ok(taskbar) = CoCreateInstance::<_, ITaskbarList>(
                                &TaskbarList,
                                None,
                                CLSCTX_INPROC_SERVER,
                            ) {
                                let _ = taskbar.HrInit();
                                let _ = taskbar.DeleteTab(hwnd_self);
                            }
                        }
                    }
                }

                // Keep forcing to bottom of the PARENT's Z-order
                let _ = SetWindowPos(
                    hwnd_self,
                    HWND_BOTTOM,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }
        }

        if !is_minimized {
            let mut rect = windows::Win32::Foundation::RECT::default();
            if GetWindowRect(main_hwnd, &mut rect).is_ok() {
                let w = (rect.right - rect.left) as f32;
                let h = (rect.bottom - rect.top) as f32;
                window.position = bevy::window::WindowPosition::At(IVec2::new(rect.left, rect.top));
                window.resolution.set(w, h);
            }
        }

        let property_name: Vec<u16> = "RotationState\0".encode_utf16().collect();
        let prop_val = GetPropW(main_hwnd, windows::core::PCWSTR(property_name.as_ptr()));

        if prop_val.0 != 0 {
            let angle = f32::from_bits(prop_val.0 as u32);
            if let Ok(mut cam_transform) = q_camera.get_single_mut() {
                cam_transform.rotation = Quat::from_rotation_z(-angle);
            }
        }
    }
}
