#[macro_use]
extern crate serde_derive;

use image::ColorType;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "webserver")]
mod hyper_adapt;
mod modutil;
mod pixelutil;
mod quat;
mod render;
mod vec3;
#[cfg(feature = "webserver")]
mod webserver;

use clap::{crate_authors, crate_version, Arg, Command};
use render::{
    render, render_frames, RenderColor, RenderEnv, RenderFloor, RenderMaterial, RenderObject,
    RenderPattern, RenderSphere, UVMap,
};
use vec3::Vec3;
#[cfg(feature = "webserver")]
use webserver::{run_webserver, ServerParams};

fn main() -> anyhow::Result<()> {
    let matches = Command::new("ray-rust")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("width")
            .help("Width of the image [px]")
            .required(true)
        )
        .arg(Arg::new("height")
            .help("Height of the image [px]")
            .required(true)
        )
        .arg(Arg::new("threads")
            .help("thread count")
            .short('t')
            .long("threads")
            .takes_value(true)
            .default_value("8")
        )
        .arg(Arg::new("output")
            .help("Output file name")
            .short('o')
            .long("output")
            .takes_value(true)
            .default_value("foo.png")
        )
        .arg(Arg::new("raymarch")
            .help("Use ray marching")
            .short('m')
            .long("raymarch")
        )
        .arg(Arg::new("gloweffect")
            .help("Enable glow effect and set its strength when ray marching method is used")
            .short('g')
            .long("gloweffect")
            .takes_value(true)
        )
        .arg(Arg::new("serialize_file")
            .help("File name for serialized scene output. If omitted, scene is not output.")
            .short('s')
            .long("serialize_file")
            .takes_value(true)
        )
        .arg(Arg::new("deserialize_file")
            .help("File name for deserialized scene input. If omitted, default scene is loaded.")
            .short('d')
            .long("deserialize_file")
            .takes_value(true)
        )
        .arg(Arg::new("webserver")
            .help("Launch a web server that responds with rendered image, rather than producing static images.
Good for interactive session.")
            .short('w')
            .long("webserver")
        )
        .arg(Arg::new("port_no")
            .help("Port number, if use web server")
            .short('p')
            .long("port_no")
            .takes_value(true)
            .default_value("3000")
        )
        .get_matches();

    fn parser<Output>(matches: &clap::ArgMatches, name: &str) -> Output
    where
        Output: FromStr + Display,
        <Output as FromStr>::Err: Display,
    {
        let ret = matches
            .value_of_t(name)
            .unwrap_or_else(|_| panic!("Parsing {} failed", name));
        println!("Value for {}: {}", name, ret);
        ret
    }

    /// Parser procedure for optional parameter
    fn parser_opt<Output>(matches: &clap::ArgMatches, name: &str) -> Option<Output>
    where
        Output: FromStr + Display,
        <Output as FromStr>::Err: Display,
    {
        let ret = matches.value_of_t(name);
        if let Ok(ref dbg) = ret {
            println!("Value for {}: {}", name, dbg);
        }
        ret.ok()
    }

    let width = parser(&matches, "width");
    let height = parser(&matches, "height");
    let thread_count = parser(&matches, "threads");
    let output: String = parser(&matches, "output");

    let use_raymarching = matches.is_present("raymarch");
    let glow_effect = parser_opt(&matches, "gloweffect");
    let serialize_file = parser_opt::<String>(&matches, "serialize_file");
    let deserialize_file = parser_opt::<String>(&matches, "deserialize_file");
    let webserver = matches.is_present("webserver");
    #[cfg(feature = "webserver")]
    let port_no = parser(&matches, "port_no");

    let xmax: usize = width/*	((XRES + 1) * 2)*/;
    let ymax: usize = height/*	((YRES + 1) * 2)*/;
    let xfov: f32 = 1.;
    let yfov: f32 = ymax as f32 / xmax as f32;

    let mut data = vec![0u8; 3 * width * height];

    for y in 0..height {
        for x in 0..width {
            data[(x + y * width) * 3] = ((x) * 255 / width) as u8;
            data[(x + y * width) * 3 + 1] = ((y) * 255 / height) as u8;
            data[(x + y * width) * 3 + 2] = ((x + y) % 32 + 32) as u8;
        }
    }

    let mut putpoint = |x: i32, y: i32, fc: &RenderColor| {
        data[(x as usize + y as usize * width) * 3] = (fc.r * 255.).min(255.) as u8;
        data[(x as usize + y as usize * width) * 3 + 1] = (fc.g * 255.).min(255.) as u8;
        data[(x as usize + y as usize * width) * 3 + 2] = (fc.b * 255.).min(255.) as u8;
    };

    let mut materials: HashMap<String, Arc<RenderMaterial>> = HashMap::new();

    let floor_material = Arc::new(
        RenderMaterial::new(
            "floor".to_string(),
            RenderColor::new(1.0, 1.0, 0.0),
            RenderColor::new(0.0, 0.0, 0.0),
            0,
            0.,
            0.0,
        )
        .pattern(RenderPattern::RepeatedGradation)
        .pattern_scale(300.)
        .pattern_angle_scale(0.2)
        .texture_ok("bar.png"),
    );
    materials.insert("floor".to_string(), floor_material);

    let mirror_material = Arc::new(
        RenderMaterial::new(
            "mirror".to_string(),
            RenderColor::new(0.0, 0.0, 0.0),
            RenderColor::new(1.0, 1.0, 1.0),
            24,
            0.,
            0.0,
        )
        .frac(RenderColor::new(1., 1., 1.)),
    );

    let red_material = Arc::new(
        RenderMaterial::new(
            "red".to_string(),
            RenderColor::new(0.8, 0.0, 0.0),
            RenderColor::new(0.0, 0.0, 0.0),
            24,
            0.,
            0.0,
        )
        .glow_dist(5.),
    );

    let transparent_material = Arc::new(
        RenderMaterial::new(
            "transparent".to_string(),
            RenderColor::new(0.0, 0.0, 0.0),
            RenderColor::new(0.0, 0.0, 0.0),
            0,
            1.,
            1.5,
        )
        .frac(RenderColor::new(1.49998, 1.49999, 1.5)),
    );

    let objects: Vec<RenderObject> = vec![
        /* Plane */
        RenderObject::Floor(
            RenderFloor::new_raw(
                materials.get("floor").unwrap().clone(),
                Vec3::new(0.0, -300.0, 0.0),
                Vec3::new(0., 1., 0.),
            )
            .uvmap(UVMap::ZX),
        ),
        // RenderFloor::new (floor_material,       Vec3::new(-300.0,   0.0,  0.0),  Vec3::new(1., 0., 0.)),
        /* Spheres */
        RenderSphere::new(mirror_material.clone(), 80.0, Vec3::new(0.0, -30.0, 172.0)),
        RenderSphere::new(mirror_material, 80.0, Vec3::new(-200.0, -30.0, 172.0)),
        RenderSphere::new(red_material, 80.0, Vec3::new(-200.0, -200.0, 172.0)),
        /*	{80.0F,  70.0F,-200.0F,150.0F, 0.0F, 0.0F, 0.8F, 0.0F, 0.0F, 0.0F, 0.0F,24, 1., 1., {1.}},*/
        RenderSphere::new(transparent_material, 100.0, Vec3::new(70.0, -200.0, 150.0)),
        /*	{000.F, 0.F, 0.F, 1500.F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 1.0F, 1.0F,24, 0, 0},*/
        /*	{100.F, -70.F, -150.F, 160.F, 0.0F, 0.5F, 0.0F, 0.0F, 0.0F, 0.0F, 0.0F,24, .5F, .2F},*/
    ];

    use std::f32::consts::PI;

    fn bgcolor(ren: &RenderEnv, direction: &Vec3) -> RenderColor {
        let phi = direction.z.atan2(direction.x);
        let the = direction.y.asin();
        let d = (50. * PI + phi * 10. * PI) % (2. * PI) - PI;
        let dd = (50. * PI + the * 10. * PI) % (2. * PI) - PI;
        let ret = RenderColor::new(
            0.5 / (15. * (d * d * dd * dd) + 1.),
            0.25 - direction.y / 4.,
            0.25 - direction.y / 4.,
        );
        let dot = ren.light.dot(direction);

        if dot > 0.9 {
            if 0.9995 < dot {
                RenderColor::new(2., 2., 2.)
            } else {
                let ret2 = if 0.995 < dot {
                    let dd = (dot - 0.995) * 150.;
                    RenderColor::new(ret.r + dd, ret.g + dd, ret.b + dd)
                } else {
                    ret
                };
                let dot2 = dot - 0.9;
                RenderColor::new(ret2.r + dot2 * 5., ret2.g + dot2 * 5., ret2.b)
            }
        } else {
            ret
        }
        // else PointMandel(dir->x * 2., dir->z * 2., 32, ret);
    }

    let mut ren: RenderEnv = RenderEnv::new(
        Vec3::new(0., -150., -300.),       /* cam */
        Vec3::new(0., -PI / 2., -PI / 2.), /* pyr */
        xmax as i32,
        ymax as i32, /* xres, yres */
        xfov,
        yfov, /* xfov, yfov*/
        //pointproc: putpoint, /* pointproc */
        bgcolor, /* bgproc */
    )
    .materials(materials)
    .objects(objects)
    .light(Vec3::new(50., 60., -50.))
    .use_raymarching(use_raymarching)
    .glow_effect(glow_effect);

    if let Some(file_name) = deserialize_file {
        let mut file = std::fs::File::open(file_name)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        ren.deserialize(&buf).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Deserialize error: ".to_string() + &e.s,
            )
        })?;
        // println!("deserialized {} materials and {} objects", ren.materials.len(), ren.objects.len());
        // for material in ren.materials.iter() {
        //     println!("  {:?}", material);
        // }
        // for (i, object) in ren.objects.iter().enumerate() {
        //     println!("  [{}]: {}", i, object.get_interface().get_material().get_name());
        // }
    }

    if webserver {
        #[cfg(feature = "webserver")]
        return Ok(run_webserver(Arc::new(ServerParams {
            width,
            height,
            thread_count,
            port_no,
            ren,
        }))?);

        #[cfg(not(feature = "webserver"))]
        return Err(anyhow::anyhow!("Web server is not enabled in build config"));
    }

    if let Some(file_name) = serialize_file {
        let mut file = std::fs::File::create(file_name)?;
        file.write_all(&ren.serialize()?.bytes().collect::<Vec<u8>>())?;
    }

    let start = Instant::now();

    let ret = if !ren.camera_motion.0.is_empty() {
        render_frames(
            &mut ren,
            width,
            height,
            &mut |i, data| {
                let frame_output = format!("{}{}.png", output, i);
                image::save_buffer(
                    frame_output,
                    &data,
                    width as u32,
                    height as u32,
                    ColorType::Rgb8,
                )
                .ok();
            },
            thread_count,
        );
        Ok(())
    } else {
        render(&ren, &mut putpoint, thread_count);

        image::save_buffer(output, &data, width as u32, height as u32, ColorType::Rgb8)
    };

    let end = start.elapsed();
    println!(
        "Rendering time: {}.{:06}",
        end.as_secs(),
        end.subsec_micros()
    );
    Ok(ret?)
}
