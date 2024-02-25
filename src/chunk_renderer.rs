
use crate::chunk_parser;
use three_d::*;

pub async fn init()
{
    println!("HASSDASD");

    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
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

    // let mut img: CpuTexture = three_d_asset::io::load_async(&["dirt.png"])
    //     .await
    //     .unwrap()
    //     .deserialize("")
    //     .unwrap();
    // img.data.to_linear_srgb();

    // let mut cube_map: TextureCubeMap = TextureCubeMap::new(&context, &img, &img, &img, &img, &img, &img);

    // let mut color_texture: ColorTexture =
    // img.width = 16;
    // img.height = 16;
    // img.wrap_t
    let mut loaded = three_d_asset::io::load_async(&[
        "src/assets/Test.png",
    ])
    .await
    .unwrap();

    // Box
    let mut cpu_texture: CpuTexture = loaded.deserialize("Test").unwrap();
    cpu_texture.data.to_linear_srgb();
    let mut box_object = Gm::new(
        Mesh::new(&context, &CpuMesh::cube()),
        ColorMaterial {
            texture: Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture)),
            ..Default::default()
        },
    );
    // box_object.material.render_states.cull = Cull::Back;
    box_object.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 0.0)) * Mat4::from_scale(0.1));

    // Lights
    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(&context, 2.0, Srgba::WHITE, &vec3(0.0, -1.0, -1.0));

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);
        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw
        if redraw {
            frame_input.screen().clear(ClearState::default()).render(
                &camera,
                box_object.into_iter(),
                &[&ambient, &directional],
            );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
    // let material = ColorMaterial {
    //     color: Srgba::WHITE,
    //     texture: Some(Texture2DRef::from_cpu_texture(&context, &img)),
    //     ..Default::default()
    // };

    // println!("HASSDASD");
    // let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    // let mut cube = Gm::new(
    //     Mesh::new(&context, &CpuMesh::cube()),
    //     material
    // );

    // cube.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, 1.3)) * Mat4::from_scale(0.5));

    // let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));

    // window.render_loop(move |mut frame_input| {
    //     camera.set_viewport(frame_input.viewport);
    //     control.handle_events(&mut camera, &mut frame_input.events);

    //     frame_input
    //         .screen()
    //         .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
    //         .render(
    //             &camera,
    //                 cube
    //                 .into_iter()
    //                 .chain(&cube),
    //                 // .chain(&axes),
    //                 // .chain(&bounding_box_cube),
    //             &[&light0],
    //         );

    //     FrameOutput::default()
    // });
}


pub fn render_chunk(chunk_blocks: Vec<chunk_parser::Block>, x: i32, z: i32)
{
    println!("ASDASD");
}