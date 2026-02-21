use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
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

#[derive(Resource, Default)]
struct TrackingState {
    hwnd: isize,
    frames: u32,
    current_rotation: u8,
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

    App::new()
        .insert_resource(ClearColor(Color::hex("030308").unwrap())) // cosmic-bg
        .insert_resource(TrackingState {
            hwnd: target_hwnd,
            frames: 0,
            current_rotation: 0,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Year 50,000 - Quantum Logo (Pure Rust)".into(),
                resolution: (width, height).into(),
                decorations: false,
                transparent: true,
                visible: false, // start invisible to hide the white flash
                position,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, animate_scene)
        .add_systems(Update, sync_window_process)
        .run();
}

// --- Scene Setup (Equivalent to window.onload scene initialization) ---
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Camera setup with Bloom (for glitch and ambient glow effect)
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true, // Enables HDR for neon bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::default(),
    ));

    // --- Logo Elements (The Quantum Core) ---

    // 1. Outer Torus (Energy Field) - Emulating TorusKnot
    let torus_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 1.0, 0.3), // 0x00ffff with opacity
        emissive: Color::rgba(0.0, 1.0, 1.0, 5.0),   // glowing cyan
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
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

    // 2. Inner Icosahedron (The Core)
    let core_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 0.0, 1.0, 0.9), // 0xff00ff fuchsia
        emissive: Color::hex("aa00ff").unwrap() * 2.0,
        metallic: 0.9,
        perceptual_roughness: 0.1,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    // In Bevy, a Sphere with low subdivisions emulates an Icosahedron
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(1.5).mesh().ico(0).unwrap()),
            material: core_material,
            ..default()
        },
        QuantumCore,
    ));

    // 3. Inner Wireframe (Data lines)
    let wire_material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.15),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(1.52).mesh().ico(1).unwrap()),
            material: wire_material,
            ..default()
        },
        CoreWireframe,
    ));

    // 4. Particle System (Orbiting Quantum Dust)
    let particles_count = 800;
    let particle_mesh = meshes.add(Sphere::new(0.05));
    let particle_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 1.0, 0.8),
        emissive: Color::rgba(0.0, 1.0, 1.0, 2.0),
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

    // --- Lighting ---
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::hex("00ffff").unwrap(),
                intensity: 2000.0,
                range: 50.0,
                ..default()
            },
            transform: Transform::from_xyz(5.0, 5.0, 5.0),
            ..default()
        },
        PointLight1,
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::hex("ff00ff").unwrap(),
            intensity: 2000.0,
            range: 50.0,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, -5.0, -5.0),
        ..default()
    });

    // --- UI Overlay Elements (Equivalent to HTML absolute divs) ---
    // A E T H E R Typography
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
                    "QUANTUM CORE . EST 50,000 AD",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::hex("40e0d0").unwrap(), // cyan-400 equivalent
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
    let window = q_window.single();

    // Interaction logic
    let mut target_x = 0.0;
    let mut target_y = 0.0;

    if let Some(cursor_position) = window.cursor_position() {
        let window_half_x = window.width() / 2.0;
        let window_half_y = window.height() / 2.0;
        let mouse_x = cursor_position.x - window_half_x;
        let mouse_y = cursor_position.y - window_half_y;

        target_x = mouse_x * 0.001;
        target_y = mouse_y * 0.001;
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

// --- Sync Window Process ---
fn sync_window_process(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut q_camera: Query<&mut Transform, With<Camera3d>>,
    mut tracking: ResMut<TrackingState>,
) {
    if let Ok(mut window) = q_window.get_single_mut() {
        tracking.frames += 1;

        #[cfg(windows)]
        {
            if tracking.hwnd != 0 {
                use windows::core::s;
                use windows::Win32::Foundation::HWND;
                use windows::Win32::System::Com::{
                    CoCreateInstance, CoInitialize, CLSCTX_INPROC_SERVER,
                };
                use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};
                use windows::Win32::UI::WindowsAndMessaging::{
                    FindWindowA, GetPropW, GetWindowRect, IsIconic,
                };

                let main_hwnd = HWND(tracking.hwnd);

                unsafe {
                    let is_minimized = IsIconic(main_hwnd).as_bool();

                    if is_minimized {
                        window.visible = false;
                    } else if tracking.frames >= 5 {
                        window.visible = true;
                    }

                    if tracking.frames == 5 {
                        let std_title = s!("Year 50,000 - Quantum Logo (Pure Rust)");
                        let hwnd_self = FindWindowA(windows::core::PCSTR::null(), std_title);
                        if hwnd_self.0 != 0 {
                            // Hide from taskbar using COM ITaskbarList
                            CoInitialize(None).ok();
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

                    if !is_minimized {
                        let mut rect = windows::Win32::Foundation::RECT::default();
                        if GetWindowRect(main_hwnd, &mut rect).is_ok() {
                            let width = (rect.right - rect.left) as f32;
                            let height = (rect.bottom - rect.top) as f32;

                            let x = rect.left;
                            let y = rect.top;

                            window.position = bevy::window::WindowPosition::At(IVec2::new(x, y));
                            window.resolution.set(width, height);
                        }
                    }

                    // Check for rotation state property
                    let mut property_name: Vec<u16> = "RotationState".encode_utf16().collect();
                    property_name.push(0);

                    let prop_val =
                        GetPropW(main_hwnd, windows::core::PCWSTR(property_name.as_ptr()));

                    if prop_val.0 != 0 || tracking.frames > 10 {
                        let angle = f32::from_bits(prop_val.0 as u32);
                        if let Ok(mut cam_transform) = q_camera.get_single_mut() {
                            // We want to rotate around Z axis to match screen rotation
                            cam_transform.rotation = Quat::from_rotation_z(-angle);
                        }
                    }
                }
            } else if tracking.frames == 5 {
                window.visible = true;
            }
        }
    }
}
