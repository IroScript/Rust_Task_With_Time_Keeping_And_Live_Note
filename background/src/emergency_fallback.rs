// Emergency fallback for systems with only Microsoft Basic Render Driver
// This version uses absolute minimal 3D to avoid DirectX crashes

use bevy::prelude::*;

fn main() {
    println!("=== EMERGENCY FALLBACK MODE ===");
    println!("Detected Microsoft Basic Render Driver");
    println!("Running minimal version to avoid crashes");
    println!("For full experience, install Intel HD Graphics 4000 drivers");
    println!("===============================\n");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Quantum Logo - Emergency Mode (Install GPU Drivers!)".into(),
                        resolution: (800.0, 600.0).into(),
                        decorations: true, // Keep decorations for emergency mode
                        transparent: false, // No transparency
                        visible: true,
                        // Hide from taskbar even in emergency mode
                        window_level: bevy::window::WindowLevel::AlwaysOnBottom,
                        present_mode: bevy::window::PresentMode::Fifo, // Most stable
                        focused: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::render::RenderPlugin {
                    render_creation: bevy::render::settings::RenderCreation::Automatic(
                        bevy::render::settings::WgpuSettings {
                            backends: Some(wgpu::Backends::all()),
                            power_preference: wgpu::PowerPreference::LowPower,
                            limits: wgpu::Limits::downlevel_defaults(), // Minimal limits
                            ..default()
                        }
                    ),
                    ..default()
                }),
        )
        .add_systems(Startup, setup_emergency_scene)
        .add_systems(Update, animate_emergency)
        .run();
}

fn setup_emergency_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Minimal camera - no HDR, no bloom
    commands.spawn(Camera3dBundle {
        camera: Camera { hdr: false, ..default() },
        tonemapping: bevy::core_pipeline::tonemapping::Tonemapping::None,
        transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Single simple cube instead of complex geometry
    let material = materials.add(StandardMaterial {
        base_color: Color::CYAN,
        emissive: Color::CYAN * 0.1, // Minimal emissive
        unlit: true, // No lighting calculations
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material,
            ..default()
        },
        EmergencyCube,
    ));

    // Minimal ambient light only
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    // Warning UI
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "⚠️ EMERGENCY MODE ⚠️",
                TextStyle {
                    font_size: 40.0,
                    color: Color::RED,
                    ..default()
                },
            ));
            parent.spawn(
                TextBundle::from_section(
                    "Microsoft Basic Render Driver Detected",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::YELLOW,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                }),
            );
            parent.spawn(
                TextBundle::from_section(
                    "Install Intel HD Graphics 4000 drivers for full experience",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                }),
            );
            parent.spawn(
                TextBundle::from_section(
                    "Run: cargo run --bin check_gpu",
                    TextStyle {
                        font_size: 14.0,
                        color: Color::GRAY,
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

#[derive(Component)]
struct EmergencyCube;

fn animate_emergency(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<EmergencyCube>>,
) {
    let elapsed = time.elapsed_seconds();
    
    for mut transform in &mut query {
        // Very slow, simple rotation to minimize GPU load
        transform.rotation = Quat::from_rotation_y(elapsed * 0.2);
    }
}