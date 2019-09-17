extern crate image;

use std::fs::File;
use std::env;
use image::png::PNGEncoder;
use image::ColorType;

mod render;

use render::{Vec3, RenderColor, floor_static, render_object_static_def, SOBJECT, RenderEnv, render};



fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let (width, height): (usize, usize) = {
        let mut width = 256;
        let mut height = 256;
        if 1 < args.len() {
            width = args[1].parse().expect("width must be an int");
        }
        if 2 < args.len() {
            height = args[2].parse().expect("height must be an int");
        }
        (width, height)
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
        // PutPointWin(&wg, x, ren.yres - y,
        //     RGB((BYTE )(fc->fred > 1.F ? 255 : fc->fred * 255),
        //         (BYTE )(fc->fgreen > 1.F ? 255 : fc->fgreen * 255),
        //         (BYTE )(fc->fblue > 1.F ? 255 : fc->fblue * 255)));
    };

    let objects: Vec<SOBJECT> = vec!{
    /* Plane */
        SOBJECT::new(&floor_static,               0.0, Vec3::new(  0.0,-300.0,  0.0), RenderColor::new(0.0, 0.5, 0.5), RenderColor::new(0.0, 0.0, 0.0),  0, 0., 0., RenderColor::new(1., 1., 1.)),
    /* Spheres */
        SOBJECT::new(&render_object_static_def,  80.0, Vec3::new(  0.0, -30.0,172.0), RenderColor::new(0.0, 0.0, 0.0), RenderColor::new(1.0, 1.0, 1.0), 24, 0., 0., RenderColor::new(1., 1., 1.)),
        SOBJECT::new(&render_object_static_def,  80.0, Vec3::new(-200.0,-200.0,172.0), RenderColor::new(0.8, 0.0, 0.0), RenderColor::new(0.0, 0.0, 0.0),24, 0., 0., RenderColor::new(1., 1., 1.)),
    /*	{&render_object_static_def,  80.0F,  70.0F,-200.0F,150.0F, 0.0F, 0.0F, 0.8F, 0.0F, 0.0F, 0.0F, 0.0F,24, 1., 1., {1.}},*/
        SOBJECT::new(&render_object_static_def, 100.0, Vec3::new(70.0,-200.0,150.0), RenderColor::new(0.0, 0.0, 0.0), RenderColor::new(0.0, 0.0, 0.0), 0, 1., 1.5, RenderColor::new(1.49998, 1.49999, 1.5)),
    /*	{&render_object_static_def, 1000.F, 0.F, 0.F, 1500.F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 1.0F, 1.0F,24, 0, 0},*/
    /*	{&render_object_static_def,  100.F, -70.F, -150.F, 160.F, 0.0F, 0.5F, 0.0F, 0.0F, 0.0F, 0.0F, 0.0F,24, .5F, .2F},*/
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

    let num_objects = objects.len();

    let mut ren: RenderEnv = RenderEnv{
        cam: Vec3::new(0., -150., -300.), /* cam */
        pyr: Vec3::new(0., -PI / 2., -PI / 2.), /* pyr */
        xres: xmax as i32,
        yres: ymax as i32, /* xres, yres */
        xfov: xfov,
        yfov: yfov, /* xfov, yfov*/
        //pointproc: putpoint, /* pointproc */
        objects,
        nobj: num_objects as i32, /* objects, nobj */
        light: Vec3::new(50., 60., -50.), /* light */
        vnm: Vec3::new(0., 1., 0.), /* vnm */
        bgproc: bgcolor, /* bgproc */
    };
    render(&mut ren, &mut putpoint);

    let buffer = File::create("foo.png")?;
    let encoder = PNGEncoder::new(buffer);

    encoder.encode(&data, width as u32, height as u32, ColorType::RGB(8))
}
