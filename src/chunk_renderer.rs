use std::path::Path;

use crate::chunk_parser;
use three_d::*;
use three_d_asset::io::RawAssets;

const TEXTURES_PATH: &str = "src/assets/textures/"; // todo, settings
const BLOCK_SCALE: f32 = 1.0;

pub struct RendererContext
{
    pub window: Window,
    pub camera: Camera,
}

impl RendererContext
{
    pub fn setup_window(title: String, window_width: u32, window_height: u32) -> Window
    {
        let window: Window = Window::new(
        WindowSettings {
            title: title,
            max_size: Some((window_width, window_height)),
            ..Default::default()
        }).unwrap();

        window
    }

    pub fn setup_camera(window: &Window,start_position: Vec3, point_target: Vec3) -> Camera
    {
        let camera: Camera = Camera::new_perspective(
            window.viewport(),
            start_position,
            point_target,
            vec3(0.0, 1.0, 0.0), // up
            degrees(45.0),       // y fov
            0.1,                 // z near
            1000.0,              // z far
        );

        camera
    }
}

pub async fn init(chunk_blocks: Vec<chunk_parser::Block>)
{
    // let window = Window::new(WindowSettings {
    //     title: "MC Viewer!".to_string(),
    //     max_size: Some((1280, 720)),
    //     ..Default::default()
    // })
    // .unwrap();

    // let context = window.gl();

    // let mut camera = Camera::new_perspective(
    //     window.viewport(),
    //     vec3(5.0, 2.0, 2.5),
    //     vec3(0.0, 0.0, -0.5),
    //     vec3(0.0, 1.0, 0.0),
    //     degrees(45.0),
    //     0.1,
    //     1000.0,
    // );

    // let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut unique_texture_list = get_chunk_textures(&chunk_blocks).await;
    let mut chunk_textures = three_d_asset::io::load_async(&unique_texture_list).await.unwrap();


    println!("{:#?}", chunk_blocks);


    // // Box
    // let mut cpu_texture: CpuTexture = loaded.deserialize("dirt").unwrap();
    // cpu_texture.data.to_linear_srgb();
    // let mut box_object = Gm::new(
    //     Mesh::new(&context, &CpuMesh::cube()),
    //     ColorMaterial {
    //         texture: Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture)),
    //         ..Default::default()
    //     },
    // );
    // // box_object.material.render_states.cull = Cull::Back;
    // box_object.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(0.1));

    // // Lights
    // let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    // let directional = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(0.0, -1.0, -1.0));

    // // main loop
    // window.render_loop(move |mut frame_input| {
    //     let mut redraw = frame_input.first_frame;
    //     redraw |= camera.set_viewport(frame_input.viewport);
    //     redraw |= control.handle_events(&mut camera, &mut frame_input.events);

    //     // draw
    //     if redraw {
    //         frame_input.screen().clear(ClearState::default()).render(
    //             &camera,
    //             box_object.into_iter(),
    //             &[&ambient, &directional],
    //         );
    //     }

    //     FrameOutput {
    //         swap_buffers: redraw,
    //         ..Default::default()
    //     }
    // });
}

// todo: GREATLY FUCKING IMPROVE. Fix the logic to be less shit andd remove the init setup. It was complaining that render context went out of scope, soemthing to do with the liftimes...
// also make pretty >:( also fix the fucking position logic god fucking damnit... lmao
pub async fn render_chunk(chunk_blocks: Vec<chunk_parser::Block>)
{
    let window = Window::new(WindowSettings {
        title: "MC Viewer!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, 2.0, 2.5),
        vec3(0.0, 0.0, -0.5),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let unique_texture_list = get_chunk_textures(&chunk_blocks).await;
    let mut chunk_textures = three_d_asset::io::load_async(&unique_texture_list).await.unwrap();
    let window_context = window.gl();

    let mut cube_array: Vec<Gm<Mesh, ColorMaterial>> = Vec::new();

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut z: f32 = 0.0;

    let block_size = BLOCK_SCALE * 2.0;

    for block in chunk_blocks
    {
        if (strip_block_name(&mut block.name.clone()) != String::from("air"))
        {
            let mut cpu_texture: CpuTexture = chunk_textures.deserialize(strip_block_name(&mut block.name.clone())).unwrap();
            cpu_texture.data.to_linear_srgb();

            let mut cube: Gm<Mesh, ColorMaterial> = Gm::new(
                Mesh::new(&window_context, &CpuMesh::cube()),
                ColorMaterial 
                {
                    texture: Some(Texture2DRef::from_cpu_texture(&window_context, &cpu_texture)),
                    ..Default::default()
                },
            );

            cube.set_transformation(Mat4::from_translation(vec3(x, y, z)) * Mat4::from_scale(BLOCK_SCALE));
            cube_array.push(cube);
        }

        x += block_size;

        if x >= 32.0
        {
            x = 0.0;
            z += block_size;
        }

        if z >= 32.0
        {
            z = 0.0;
            y += block_size;
        }
    }   

        window.render_loop(move |mut frame_input| {
            let mut redraw = frame_input.first_frame;
            redraw |= camera.set_viewport(frame_input.viewport);
            redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw
        if redraw {
            frame_input.screen().clear(ClearState::default()).render(
                &camera,
                cube_array.iter().flat_map(|it| it), // blocks are consumed
                &[],
            );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}

async fn get_chunk_textures(chunk_blocks: &Vec<chunk_parser::Block>) -> Vec<String>
{
    let mut texture_load_list: Vec<String> = Vec::new(); 
    let unique_chunk_block_names = get_unique_blocks(&chunk_blocks);

    for block_name in unique_chunk_block_names
    {
        let file_extension: String = String::from(".png"); // tmp
        let file_path = format!("{}{}{}", TEXTURES_PATH, block_name, file_extension);

        if (Path::new(&file_path).exists())
        {
            texture_load_list.push(file_path)
        } else
        {
            texture_load_list.push(String::from("D:/Projects2023/Rust/mc_viewer/src/assets/textures/bedrock.png"))
        }
    }

    return texture_load_list;
}

fn get_unique_blocks(chunk_blocks: &Vec<chunk_parser::Block>) -> Vec<String>
{
    let mut unique_names: Vec<String> = Vec::new();
    let illegal_names: Vec<String> = vec![String::from("minecraft:air")];

    for block in chunk_blocks
    {
        if !unique_names.contains(&block.name) && !illegal_names.contains(&block.name)
        {
            let mut block_stripped_name = strip_block_name(&mut block.name.clone());
            unique_names.push(block_stripped_name);
        }
    }

    return unique_names
}

fn strip_block_name(block_name: &mut String) -> String
{
    return block_name.replace("minecraft:", "");
}
