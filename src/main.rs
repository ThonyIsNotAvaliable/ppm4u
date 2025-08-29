use bevy::{
    asset::RenderAssetUsages,
    input::mouse::MouseWheel,
    prelude::*,
    reflect::List,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};


use std::fs;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ppm4u".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(Time::<Fixed>::from_hz(5.0))
        .insert_resource(ZoomScale::default())
        .add_event::<UpdateImg>()
        .add_plugins(EguiPlugin::default())
        .insert_resource(FilePath::default())
        .add_systems(Startup, (setup_camera_system, setup))
        .add_systems(Update, (zoom, update_img, viewer_input))
        .add_systems(EguiPrimaryContextPass, app_ui)
        .run();
}

#[derive(Component)]
struct ImgComponent;

#[derive(Resource)]
struct ZoomScale(f32);

impl Default for ZoomScale {
    fn default() -> Self {
        ZoomScale(1.0)
    }
}

#[derive(Resource, Default)]
struct FilePath(String);

#[derive(Event)]
pub struct UpdateImg;

struct PPMfile {
    width: u64,
    height: u64,
    contents: Vec<u8>,
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn app_ui(mut contexts: EguiContexts, mut path: ResMut<FilePath>) -> Result {
    egui::Window::new("Commands").show(contexts.ctx_mut()?, |ui| {
        ui.label("Aperte espaço para atualizar a imagem");
        ui.text_edit_singleline(&mut path.0);
        ui.add_space(10.0);
    });
    Ok(())
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Define image dimensions
    let width = 64;
    let height = 64;

    // Create a Vec<u8> for pixel data (e.g., a simple red square)
    let mut pixel_data = vec![0; width * height * 4]; // RGBA8Unorm
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 4;
            pixel_data[index] = 255; // Red
            pixel_data[index + 1] = 0; // Green
            pixel_data[index + 2] = 0; // Blue
            pixel_data[index + 3] = 255; // Alpha
        }
    }

    // Create the Image asset
    let image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixel_data,
        TextureFormat::Rgba8Unorm,       // Or another suitable format
        RenderAssetUsages::RENDER_WORLD, // Add other Image settings if needed, like sampler
    );

    // Add the Image to assets and get its handle
    let image_handle = images.add(image);

    // Spawn a sprite using the created texture
    commands
        .spawn(Sprite::from_image(image_handle))
        .insert(ImgComponent);
}

fn update_img(
    mut images: ResMut<Assets<Image>>,
    path: Res<FilePath>,
    mut query: Query<(&ImgComponent, &mut Sprite)>,
    mut events: EventReader<UpdateImg>,
) {
    for _ in events.read() {
        let data: PPMfile = match read_from_ppm(&path.0) {
            Ok(d) => d,
            Err(_) => PPMfile {
                width: 64,
                height: 64,
                contents: {
                    let t = vec![0; 64 * 64 * 3];
                    //t.fill(255);
                    t
                },
            },
        };
        // Define image dimensions
        let width = data.width as usize;
        let height = data.height as usize;

        // Create a Vec<u8> for pixel data (e.g., a simple red square)
        let mut pixel_data = vec![0; width * height * 4]; // RGBA8Unorm
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) * 4;
                let kindex = (y * width + x) * 3;
                pixel_data[index] = data.contents[kindex]; // Red
                pixel_data[index + 1] = data.contents[kindex + 1]; // Green
                pixel_data[index + 2] = data.contents[kindex + 2]; // Blue
                pixel_data[index + 3] = 255; // Alpha
            }
        }

        // Create the Image asset
        let image = Image::new(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            pixel_data,
            TextureFormat::Rgba8Unorm,       // Or another suitable format
            RenderAssetUsages::RENDER_WORLD, // Add other Image settings if needed, like sampler
        );

        // Add the Image to assets and get its handle
        let image_handle = images.add(image);

        // Spawn a sprite using the created texture
        let (_, mut sprite) = query.single_mut().unwrap();
        sprite.image = image_handle;
    }
}

fn zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<(&ImgComponent, &mut Transform)>,
    buttons: Res<ButtonInput<KeyCode>>,
    mut scale: ResMut<ZoomScale>,
) {
    let (_, mut img) = query.single_mut().expect("Cam does not exist");
    for event in mouse_wheel_events.read() {
        if buttons.pressed(KeyCode::ControlLeft) && !buttons.pressed(KeyCode::ShiftLeft) {
            img.translation.x += event.y * 10.0;
        } else if !buttons.pressed(KeyCode::ControlLeft) && buttons.pressed(KeyCode::ShiftLeft) {
            img.translation.y += event.y * 10.0;
        } else {
            scale.0 = scale.0 + event.y * 0.05;
            img.scale = Vec3::ONE * scale.0;
        }
    }
    if buttons.just_pressed(KeyCode::KeyC) {
        img.translation = Vec3::ZERO;
    }
}

fn viewer_input(mut writer: EventWriter<UpdateImg>, input: Res<ButtonInput<KeyCode>>) {
    if input.pressed(KeyCode::Space) {
        writer.write(UpdateImg);
    }
}

fn read_from_ppm(path: &str) -> Result<PPMfile> {
    let mut result: Vec<String> = Vec::new();

    for line in fs::read_to_string(path)?.lines() {
        result.push(line.to_string());
    }

    let size = result[1].split(" ").collect::<Vec<&str>>();

    let pixels = result[3]
        .split(" ")
        .map(|x| x.parse().unwrap_or_default())
        .collect::<Vec<u8>>();

    Ok(PPMfile {
        width: size[0].parse()?,
        height: size[1].parse()?,
        contents: pixels,
    })
}
