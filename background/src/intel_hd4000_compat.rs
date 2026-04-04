// Intel HD Graphics 4000 compatibility version
// Avoids complex PBR shaders that cause compilation errors

use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
struct OuterTorus;
#[derive(Component)]
struct QuantumCore;
#[derive(Component)]
struct QuantumParticle;
// FpsText removed to fix unused warning

#[derive(Resource, Default)]
struct TrackingState {
    hwnd: isize,
    frames: u32,
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

    println!("=== INTEL HD GRAPHICS 4000 COMPATIBILITY MODE ===");
    println!("Using simplified rendering to avoid shader compilation errors");
    println!("==================================================");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TrackingState {
            hwnd: target_hwnd,
            frames: 0,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Quantum Logo — Intel HD4000 Compatibility".into(),
                        resolution: (width, height).into(),
                        decorations: false,
                        transparent: false, // No transparency for compatibility
                        visible: true,
                        position,
                        // Hide from taskbar
                        window_level: bevy::window::WindowLevel::AlwaysOnBottom,
                        present_mode: bevy::window::PresentMode::Fifo,
                        focused: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        bevy::render::settings::WgpuSettings {
                            backends: Some(wgpu::Backends::GL), // Force OpenGL
                            power_preference: wgpu::PowerPreference::HighPerformance,
                            limits: wgpu::Limits::downlevel_defaults(), // Use minimal limits
                            ..default()
                        },
                    ),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_simple_scene)
        .add_systems(Update, animate_simple_scene)
        .add_systems(Update, sync_window_process)
        .run();
}

fn setup_simple_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Basic camera - no HDR, no bloom, no complex features
    commands.spawn(Camera3dBundle {
        camera: Camera {
            hdr: false,
            ..default()
        },
        tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::None,
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Simple torus with unlit material to avoid PBR shader issues
    let torus_material = materials.add(StandardMaterial {
        base_color: Color::CYAN,
        emissive: Color::CYAN * 0.2,
        unlit: true,                   // CRITICAL: Avoid complex PBR shaders
        alpha_mode: AlphaMode::Opaque, // No transparency
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

    // Simple core sphere - use UV sphere instead of icosphere
    let core_material = materials.add(StandardMaterial {
        base_color: Color::FUCHSIA,
        emissive: Color::FUCHSIA * 0.3,
        unlit: true, // CRITICAL: Avoid complex PBR shaders
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(1.5).mesh().uv(16, 8)), // Simple UV sphere
            material: core_material,
            ..default()
        },
        QuantumCore,
    ));

    // Minimal particles - only 20 for compatibility
    let particle_mesh = meshes.add(Sphere::new(0.05).mesh().uv(4, 4)); // Very simple spheres
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::CYAN,
        emissive: Color::CYAN * 0.1,
        unlit: true, // CRITICAL: Avoid complex PBR shaders
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    commands
        .spawn((SpatialBundle::default(), QuantumParticle))
        .with_children(|parent| {
            for _ in 0..20 {
                // Minimal particle count
                let radius = 6.0 + rand::random::<f32>() * 2.0;
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

    // Simple ambient light only - no point lights to avoid shader complexity
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    // Simple UI
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
                    "INTEL HD GRAPHICS 4000 COMPATIBILITY MODE",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::CYAN,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                }),
            );
        });
}

fn animate_simple_scene(
    time: Res<Time>,
    q_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut q_torus: Query<
        &mut Transform,
        (
            With<OuterTorus>,
            Without<QuantumCore>,
            Without<QuantumParticle>,
        ),
    >,
    mut q_core: Query<
        &mut Transform,
        (
            With<QuantumCore>,
            Without<OuterTorus>,
            Without<QuantumParticle>,
        ),
    >,
    mut q_particles: Query<
        &mut Transform,
        (
            With<QuantumParticle>,
            Without<OuterTorus>,
            Without<QuantumCore>,
        ),
    >,
) {
    let elapsed = time.elapsed_seconds();
    let Ok(window) = q_window.get_single() else {
        return;
    };

    let mut target_x = 0.0;
    let mut target_y = 0.0;

    if let Some(cursor_position) = window.cursor_position() {
        let window_half_x = window.width() / 2.0;
        let window_half_y = window.height() / 2.0;
        target_x = (cursor_position.x - window_half_x) * 0.001;
        target_y = (cursor_position.y - window_half_y) * 0.001;
    }

    // Simple torus rotation
    if let Ok(mut transform) = q_torus.get_single_mut() {
        transform.rotate_x(0.005);
        transform.rotate_y(0.01);

        let diff_x = target_y - transform.rotation.x;
        transform.rotation *= Quat::from_rotation_x(0.05 * diff_x);
        let diff_y = target_x - transform.rotation.y;
        transform.rotation *= Quat::from_rotation_y(0.05 * diff_y);
    }

    // Simple core rotation and scaling
    let scale_val = 1.0 + (elapsed * 2.0).sin() * 0.1;
    let scale = Vec3::splat(scale_val);

    if let Ok(mut transform) = q_core.get_single_mut() {
        transform.rotate_x(-0.008);
        transform.rotate_y(-0.008);
        transform.scale = scale;
    }

    // Simple particle rotation
    if let Ok(mut transform) = q_particles.get_single_mut() {
        transform.rotation =
            Quat::from_rotation_y(elapsed * 0.05) * Quat::from_rotation_z(elapsed * 0.02);
    }
}

// Click-through hook: Returns HTTRANSPARENT for WM_NCHITTEST to pass mouse events to windows below
unsafe extern "system" fn click_through_wndproc_intel(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::WM_NCHITTEST;

    if msg == WM_NCHITTEST {
        // Return HTTRANSPARENT to make window click-through (value = -1)
        return windows::Win32::Foundation::LRESULT(-1isize);
    }

    use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

static mut ORIGINAL_WNDPROC_INTEL: Option<windows::Win32::UI::WindowsAndMessaging::WNDPROC> = None;

// Simplified window sync - same as main version but without complex features
fn sync_window_process(
    mut q_window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    mut tracking: ResMut<TrackingState>,
) {
    let Ok(mut window) = q_window.get_single_mut() else {
        return;
    };
    tracking.frames += 1;

    #[cfg(target_os = "windows")]
    {
        if tracking.hwnd != 0 {
            // Same Windows sync logic as main version
            use windows::Win32::Foundation::HWND;
            use windows::Win32::System::Com::{
                CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER,
            };
            use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};
            use windows::Win32::UI::WindowsAndMessaging::{
                FindWindowW, GetPropW, GetWindowRect, IsIconic,
            };

            let main_hwnd = HWND(tracking.hwnd);

            unsafe {
                let is_minimized = IsIconic(main_hwnd).as_bool();
                window.visible = !is_minimized && tracking.frames >= 5;
                if tracking.frames >= 5 {
                    // Use wide string for FindWindowW to be safe
                    let title_wide: Vec<u16> = "Quantum Logo — Intel HD4000 Compatibility\0"
                        .encode_utf16()
                        .collect();
                    let hwnd_self = FindWindowW(windows::core::PCWSTR(title_wide.as_ptr()), None);

                    if hwnd_self.0 != 0 {
                        use windows::Win32::UI::WindowsAndMessaging::{
                            GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_EXSTYLE, HWND_BOTTOM,
                            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED,
                            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
                        };

                        // Force window to be ToolWindow + Transparent + ALWAYS AT BOTTOM + CLICK-THROUGH
                        // Try WITHOUT WS_EX_LAYERED to see if that helps with mouse events
                        let ex_style = GetWindowLongW(hwnd_self, GWL_EXSTYLE);
                        let target_style = ex_style
                            | WS_EX_TOOLWINDOW.0 as i32
                            | WS_EX_NOACTIVATE.0 as i32
                            | WS_EX_TRANSPARENT.0 as i32;
                        if ex_style != target_style {
                            let _ = SetWindowLongW(hwnd_self, GWL_EXSTYLE, target_style);
                        }

                        // CRITICAL FIX: Install window hook to return HTTRANSPARENT for WM_NCHITTEST
                        use windows::Win32::UI::WindowsAndMessaging::{
                            GetWindowLongPtrW, SetWindowLongPtrW, GWLP_WNDPROC,
                        };

                        let current_wndproc = GetWindowLongPtrW(hwnd_self, GWLP_WNDPROC);
                        if current_wndproc != 0 && unsafe { ORIGINAL_WNDPROC_INTEL.is_none() } {
                            unsafe {
                                ORIGINAL_WNDPROC_INTEL = Some(std::mem::transmute(current_wndproc));
                                let _: isize = SetWindowLongPtrW(
                                    hwnd_self,
                                    GWLP_WNDPROC,
                                    click_through_wndproc_intel as *const () as isize,
                                );
                            }
                        }

                        // Keep forcing to bottom every frame
                        let _ = SetWindowPos(
                            hwnd_self,
                            HWND_BOTTOM,
                            0,
                            0,
                            0,
                            0,
                            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                        );

                        // Explicitly remove from taskbar (only once)
                        if tracking.frames == 5 && CoInitialize(None).is_ok() {
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

                if !is_minimized {
                    let mut rect = windows::Win32::Foundation::RECT::default();
                    if GetWindowRect(main_hwnd, &mut rect).is_ok() {
                        let w = (rect.right - rect.left) as f32;
                        let h = (rect.bottom - rect.top) as f32;
                        window.position =
                            bevy::window::WindowPosition::At(IVec2::new(rect.left, rect.top));
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
    }
}
