extern crate image;

use std::fs::File;
use image::png::PNGEncoder;
use image::ColorType;

mod render;

use render::{POS3D, fcolor_t, FCOLOR, floor_static, render_object_static_def, SOBJECT, render_env, render};

const WIDTH: usize = 64;
const HEIGHT: usize = 64;

const XRES: usize = WIDTH / 2/*WIDTH*//*320*/;
const YRES: usize = HEIGHT / 2/*HEIGHT*//*200*/;
const XMAX: usize = WIDTH/*	((XRES + 1) * 2)*/;
const YMAX: usize = HEIGHT/*	((YRES + 1) * 2)*/;
const XFOV: f32 = 1.;
const YFOV: f32 = (YMAX as f32 / XMAX as f32);


fn main() -> std::io::Result<()> {

    let mut data: [u8; (3 * WIDTH * HEIGHT)] = [0u8; 3 * WIDTH * HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            data[(x + y * WIDTH) * 3 + 0] = ((x + y) % 64) as u8;
            data[(x + y * WIDTH) * 3 + 1] = ((x + y) % 64 + 32) as u8;
            data[(x + y * WIDTH) * 3 + 2] = ((x + y) % 32 + 32) as u8;
        }
    }

    let mut putpoint = |x: i32, y: i32, fc: &FCOLOR| {
        data[(x as usize + y as usize * WIDTH) * 3 + 0] = (fc.fred * 255.) as u8;
        data[(x as usize + y as usize * WIDTH) * 3 + 1] = (fc.fgreen * 255.) as u8;
        data[(x as usize + y as usize * WIDTH) * 3 + 2] = (fc.fblue * 255.) as u8;
        // PutPointWin(&wg, x, ren.yres - y,
        //     RGB((BYTE )(fc->fred > 1.F ? 255 : fc->fred * 255),
        //         (BYTE )(fc->fgreen > 1.F ? 255 : fc->fgreen * 255),
        //         (BYTE )(fc->fblue > 1.F ? 255 : fc->fblue * 255)));
    };

    let objects: Vec<SOBJECT> = vec!{
    /* Plane */
        SOBJECT::new(&floor_static,               0.0, POS3D::new(  0.0,-300.0,  0.0), 0.0, 0.5, 0.5, 0.0, 0.0, 0.0,  0, 0., 0., fcolor_t::new(1., 1., 1.)),
    /* Spheres */
        SOBJECT::new(&render_object_static_def,  80.0, POS3D::new(  0.0, -30.0,172.0), 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 24, 0., 0., fcolor_t::new(1., 1., 1.)),
        SOBJECT::new(&render_object_static_def,  80.0, POS3D::new(-200.0,-200.0,172.0), 0.8, 0.0, 0.0, 0.0, 0.0, 0.0,24, 0., 0., fcolor_t::new(1., 1., 1.)),
    /*	{&render_object_static_def,  80.0F,  70.0F,-200.0F,150.0F, 0.0F, 0.0F, 0.8F, 0.0F, 0.0F, 0.0F, 0.0F,24, 1., 1., {1.}},*/
        SOBJECT::new(&render_object_static_def, 100.0, POS3D::new(70.0,-200.0,150.0), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 1., 1.5, fcolor_t::new(1.49998, 1.49999, 1.5)),
    /*	{&render_object_static_def, 1000.F, 0.F, 0.F, 1500.F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 1.0F, 1.0F,24, 0, 0},*/
    /*	{&render_object_static_def,  100.F, -70.F, -150.F, 160.F, 0.0F, 0.5F, 0.0F, 0.0F, 0.0F, 0.0F, 0.0F,24, .5F, .2F},*/
    };

    fn bgcolor(pos: &POS3D, fcolor: &mut fcolor_t){
        *fcolor = fcolor_t::new(0., 0., 0.);
    }

    let num_objects = objects.len();

    use std::f32::consts::PI;
    let mut ren: render_env = render_env{
        cam: POS3D::new(0., -150., -600.), /* cam */
        pyr: POS3D::new(0., -PI / 2., -PI / 2.), /* pyr */
        xres: XMAX as i32,
        yres: YMAX as i32, /* xres, yres */
        xfov: XFOV,
        yfov: YFOV, /* xfov, yfov*/
        //pointproc: putpoint, /* pointproc */
        objects,
        nobj: num_objects as i32, /* objects, nobj */
        light: POS3D::new(50., 60., -50.), /* light */
        vnm: POS3D::new(0., 1., 0.), /* vnm */
        bgproc: bgcolor, /* bgproc */
    };
    render(&mut ren, &mut putpoint);

    let buffer = File::create("foo.png")?;
    let encoder = PNGEncoder::new(buffer);

    encoder.encode(&data, WIDTH as u32, HEIGHT as u32, ColorType::RGB(8))
}
