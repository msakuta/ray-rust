extern crate image;
extern crate crossbeam;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::fs::File;
use std::env;
use std::time::Instant;
use std::io::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use image::png::PNGEncoder;
use image::ColorType;

mod render;
mod vec3;

use render::{RenderColor,
    UVMap,
    RenderMaterial, RenderPattern,
    RenderObject, RenderSphere, RenderFloor,
    RenderEnv, render};
use vec3::Vec3;


fn main() -> std::io::Result<()> {

    if env::args().len() <= 1 {
        println!("usage: {} [width] [height] [-o output] [-t thread_count] [-m] [-g glow_dist] [-s serialized.yaml] [-d serialized.yaml]", env::args().nth(0).unwrap());
        return Ok(());
    }

    let (width, height, output, thread_count, use_raymarching, use_glow_effect, glow_effect, serialize_file, deserialize_file):
        (usize, usize, String, i32, bool, bool, f32, String, String) = {
        let mut width = 640;
        let mut width_set = false;
        let mut height = 480;
        let mut height_set = false;
        let mut output = "foo.png".to_string();
        let mut use_raymarching = false;
        let mut use_glow_effect = false;
        let mut glow_effect = 1.;
        let mut thread_count = 8;
        let mut serialize_file = "".to_string();
        let mut deserialize_file = "".to_string();
        #[derive(PartialEq)]
        enum Next{Default, Output, ThreadCount, GlowEffect, SerializeFile, DeserializeFile}
        let mut next = Next::Default;
        for arg in env::args().skip(1) {
            match next {
                Next::Output => {
                    output = arg.clone();
                    next = Next::Default;
                },
                Next::ThreadCount => {
                    thread_count = arg.parse().expect("thread count must be an int");
                    assert!(1 <= thread_count);
                    next = Next::Default;
                },
                Next::GlowEffect => {
                    glow_effect = arg.parse().expect("thread count must be an int");
                    next = Next::Default;
                },
                Next::SerializeFile => {
                    serialize_file = arg;
                    next = Next::Default;
                }
                Next::DeserializeFile => {
                    deserialize_file = arg;
                    next = Next::Default;
                }
                Next::Default => {
                    if arg == "-o" {
                        next = Next::Output;
                    }
                    else if arg == "-t" {
                        next = Next::ThreadCount;
                    }
                    else if arg == "-m" {
                        use_raymarching = true;
                    }
                    else if arg == "-g" {
                        use_glow_effect = true;
                        next = Next::GlowEffect;
                    }
                    else if arg == "-s" {
                        next = Next::SerializeFile;
                    }
                    else if arg == "-d" {
                        next = Next::DeserializeFile;
                    }
                    else if !width_set {
                        width = arg.parse().expect("width must be an int");
                        width_set = true;
                    }
                    else if !height_set {
                        height = arg.parse().expect("height must be an int");
                        height_set = true;
                    }
                }
            }
        }
        (width, height, output, thread_count, use_raymarching, use_glow_effect, glow_effect, serialize_file, deserialize_file)
    };

    let xmax: usize = width/*	((XRES + 1) * 2)*/;
    let ymax: usize = height/*	((YRES + 1) * 2)*/;
    let xfov: f32 = 1.;
    let yfov: f32 = ymax as f32 / xmax as f32;

    let mut data = vec![0u8; 3 * width * height];

    for y in 0..height {
        for x in 0..width {
            data[(x + y * width) * 3 + 0] = ((x) * 255 / width) as u8;
            data[(x + y * width) * 3 + 1] = ((y) * 255 / height) as u8;
            data[(x + y * width) * 3 + 2] = ((x + y) % 32 + 32) as u8;
        }
    }

    let mut putpoint = |x: i32, y: i32, fc: &RenderColor| {
        data[(x as usize + y as usize * width) * 3 + 0] = (fc.r * 255.).min(255.) as u8;
        data[(x as usize + y as usize * width) * 3 + 1] = (fc.g * 255.).min(255.) as u8;
        data[(x as usize + y as usize * width) * 3 + 2] = (fc.b * 255.).min(255.) as u8;
    };

    let mut materials: HashMap<String, Arc<RenderMaterial>> = HashMap::new();

    let floor_material = Arc::new(RenderMaterial::new("floor".to_string(),
        RenderColor::new(1.0, 1.0, 0.0), RenderColor::new(0.0, 0.0, 0.0),  0, 0., 0.0)
        .pattern(RenderPattern::RepeatedGradation)
        .pattern_scale(300.));
    materials.insert("floor".to_string(), floor_material.clone());

    let mirror_material = Arc::new(RenderMaterial::new("mirror".to_string(),
        RenderColor::new(0.0, 0.0, 0.0), RenderColor::new(1.0, 1.0, 1.0), 24, 0., 0.0)
        .frac(RenderColor::new(1., 1., 1.)));

    let red_material = Arc::new(RenderMaterial::new("red".to_string(),
        RenderColor::new(0.8, 0.0, 0.0), RenderColor::new(0.0, 0.0, 0.0), 24, 0., 0.0)
        .glow_dist(5.));

    let transparent_material = Arc::new(RenderMaterial::new("transparent".to_string(),
        RenderColor::new(0.0, 0.0, 0.0), RenderColor::new(0.0, 0.0, 0.0),  0, 1., 1.5)
        .frac(RenderColor::new(1.49998, 1.49999, 1.5)));

    let objects: Vec<RenderObject> = vec!{
    /* Plane */
        RenderObject::Floor(
            RenderFloor::new_raw(materials.get("floor").unwrap().clone(),       Vec3::new(  0.0, -300.0,  0.0),  Vec3::new(0., 1., 0.))
            .uvmap(UVMap::ZX),
        ),
        // RenderFloor::new (floor_material,       Vec3::new(-300.0,   0.0,  0.0),  Vec3::new(1., 0., 0.)),
    /* Spheres */
        RenderSphere::new(mirror_material.clone(), 80.0, Vec3::new(   0.0, -30.0,172.0)),
        RenderSphere::new(mirror_material, 80.0, Vec3::new(   -200.0, -30.0,172.0)),
        RenderSphere::new(red_material, 80.0, Vec3::new(-200.0,-200.0,172.0)),
    /*	{80.0F,  70.0F,-200.0F,150.0F, 0.0F, 0.0F, 0.8F, 0.0F, 0.0F, 0.0F, 0.0F,24, 1., 1., {1.}},*/
        RenderSphere::new(transparent_material, 100.0, Vec3::new(  70.0,-200.0,150.0)),
    /*	{000.F, 0.F, 0.F, 1500.F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 1.0F, 1.0F,24, 0, 0},*/
    /*	{100.F, -70.F, -150.F, 160.F, 0.0F, 0.5F, 0.0F, 0.0F, 0.0F, 0.0F, 0.0F,24, .5F, .2F},*/
    };

    use std::f32::consts::PI;

    fn bgcolor(ren: &RenderEnv, direction: &Vec3) -> RenderColor{
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
            }
            else {
                let ret2 = if 0.995 < dot {
                    let dd = (d - 0.995) * 150.;
                    RenderColor::new(ret.r + dd, ret.g + dd, ret.b + dd)
                } else { ret };
                let dot2 = dot - 0.9;
                RenderColor::new(ret2.r + dot2 * 5., ret2.g + dot2 * 5., ret2.b)
            }
        }
        else {
            ret
        }
        // else PointMandel(dir->x * 2., dir->z * 2., 32, ret);
    }

    let mut ren: RenderEnv = RenderEnv::new(
        Vec3::new(0., -150., -300.), /* cam */
        Vec3::new(0., -PI / 2., -PI / 2.), /* pyr */
        xmax as i32,
        ymax as i32, /* xres, yres */
        xfov,
        yfov, /* xfov, yfov*/
        //pointproc: putpoint, /* pointproc */
        materials,
        objects,
        bgcolor, /* bgproc */
    ).light(Vec3::new(50., 60., -50.))
    .use_raymarching(use_raymarching)
    .use_glow_effect(use_glow_effect, glow_effect);

    if deserialize_file != "" {
        let mut file = std::fs::File::open(deserialize_file)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        ren.deserialize(&buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
            "Deserialize error: ".to_string() + &e.s))?;
        // println!("deserialized {} materials and {} objects", ren.materials.len(), ren.objects.len());
        // for material in ren.materials.iter() {
        //     println!("  {:?}", material);
        // }
        // for (i, object) in ren.objects.iter().enumerate() {
        //     println!("  [{}]: {}", i, object.get_interface().get_material().get_name());
        // }
    }

    if serialize_file != "" {
        let mut file = std::fs::File::create(serialize_file)?;
        file.write_all(&ren.serialize()?.bytes().collect::<Vec<u8>>())?;
    }

    let start = Instant::now();

    render(&mut ren, &mut putpoint, thread_count);

    let end = start.elapsed();
    println!("Rendering time: {}.{:06}", end.as_secs(), end.subsec_nanos() / 1_000);

    let buffer = File::create(output)?;
    let encoder = PNGEncoder::new(buffer);

    encoder.encode(&data, width as u32, height as u32, ColorType::RGB(8))
}
