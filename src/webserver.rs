extern crate tokio;

use crate::render::{render, RenderEnv, RenderColor};
use std::sync::Arc;
use tokio::prelude::*;
use tokio::runtime::Runtime;
use crate::Vec3;
use crate::quat::Quat;

use {
    hyper::{
        // Miscellaneous types from Hyper for working with HTTP.
        Body, Request, Response, Server, StatusCode, Error,

        // This function turns a closure which returns a future into an
        // implementation of the the Hyper `Service` trait, which is an
        // asynchronous function from a generic `Request` to a `Response`.
        service::service_fn,
        service::make_service_fn,

        // A function which runs a future to completion using the Hyper runtime.
        // rt::run,
    },
    std::net::SocketAddr,
};
use std::collections::HashMap;

pub struct RenderParamStruct{
    pub width: usize,
    pub height: usize,
    pub thread_count: i32,
    pub ren: RenderEnv,
}

pub type RenderParams = Arc<RenderParamStruct>;

fn render_web(ren: &RenderParamStruct) -> Vec<u8>{
    let (width, height) = (ren.width, ren.height);
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

    render(&ren.ren, &mut putpoint, ren.thread_count);
    data
}

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

async fn serve_req(req: Request<Body>/*, ren: RenderParams*/) -> Result<Response<Body>, hyper::Error> {
    // Always return successfully with a response containing a body with
    // a friendly greeting ;)
    println!("Got request at {:?}", req.uri());
    if req.uri() == "/" {
        Ok(Response::new(Body::from("<html>
        <head>
            <title>ray-rust</title>
            <script>
            window.onload = function(){
                var im = document.getElementById('render');
                var x = -100;
                var z = -100;
                var yaw = -90;
                function updatePos(){
                    im.src = `/render?x=${x}&z=${z}&y=${yaw}`;
                }
                im.onclick = function(){
                    z += 10;
                    updatePos();
                }
                updatePos();
                window.onkeydown = function(e){
                    if(event.key === 'a'){
                        x += 10 * Math.sin(yaw * Math.PI / 180);
                        z += 10 * Math.cos(yaw * Math.PI / 180);
                        updatePos();
                    }
                    else if(event.key === 'd'){
                        x -= 10 * Math.sin(yaw * Math.PI / 180);
                        z -= 10 * Math.cos(yaw * Math.PI / 180);
                        updatePos();
                    }
                    else if(event.key === 'w'){
                        x += 10 * Math.cos(yaw * Math.PI / 180);
                        z -= 10 * Math.sin(yaw * Math.PI / 180);
                        updatePos();
                    }
                    else if(event.key === 's'){
                        x -= 10 * Math.cos(yaw * Math.PI / 180);
                        z += 10 * Math.sin(yaw * Math.PI / 180);
                        updatePos();
                    }
                    else if(event.key === 'ArrowRight'){
                        yaw += 10;
                        updatePos();
                    }
                    else if(event.key === 'ArrowLeft'){
                        yaw -= 10;
                        updatePos();
                    }
                }
            }
            </script>
        </head>
        <body>
            <h1>hello, world!</h1>
            <img id='render'>
        </body>")))
    }
    else if req.uri() == "/image" {
        // render_web(&ren);
        if let Ok(mut image) = tokio::fs::File::open("barb.png").await {
            let mut buf: Vec<u8> = vec![];
            if let Ok(_) = image.read_to_end(&mut buf).await {
                println!("Responding with image {}", buf.len());
                Ok(Response::new(Body::from(buf)))
            }
            else{
                Ok(Response::new(Body::from("Error reading barb.png")))
            }
        }
        else {
            Ok(Response::new(Body::from("image")))
        }
    }
    else if req.uri().path() == "/render" {
        println!("GET /render, query = {:?}", req.uri().query());
        let (xpos, zpos, yaw) = if let Some(query) = req.uri().query() {
            let (mut xpos, mut zpos, mut yaw) = (0f32, 0f32, 0f32);
            for s in query.split("&") {
                match &s[..2] {
                    "x=" => if let Ok(f) = s[2..].parse::<f32>() {
                        xpos = f;
                    }
                    "z=" => if let Ok(f) = s[2..].parse::<f32>() {
                        zpos = f;
                    }
                    "y=" => if let Ok(f) = s[2..].parse::<f32>() {
                        yaw = f;
                    }
                    _ => ()
                }
            }
            (xpos, zpos, yaw)
        }
        else {
            (0., 0., 0.)
        };
        println!("Rendering with xpos={}", xpos);
        let deserialize_file = "out2.yaml";
        let mut file = tokio::fs::File::open(deserialize_file).await.unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).await.unwrap();

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
                        let dd = (dot - 0.995) * 150.;
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

        let xmax: usize = WIDTH;
        let ymax: usize = HEIGHT;
        let xfov: f32 = 1.;
        let yfov: f32 = ymax as f32 / xmax as f32;
    
        let mut renparam = RenderParamStruct{
            width: WIDTH,
            height: HEIGHT,
            thread_count: 8,
            ren: RenderEnv::new(
                Vec3::new(0., -150., -300.), /* cam */
                Vec3::new(0., -PI / 2., -PI / 2.), /* pyr */
                xmax as i32,
                ymax as i32,
                xfov,
                yfov,
                HashMap::new(),
                vec![],
                bgcolor,
            ).light(Vec3::new(50., 60., -50.))
        };
        renparam.ren.deserialize(&buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
            "Deserialize error: ".to_string() + &e.s)).unwrap();
        renparam.ren.camera.position.x = xpos;
        renparam.ren.camera.position.z = zpos;
        renparam.ren.camera.pyr.y = yaw * PI / 180.;
        renparam.ren.camera.rotation = Quat::from_pyr(&renparam.ren.camera.pyr);
        let imbuf = image::DynamicImage::ImageRgb8(image::ImageBuffer::from_raw(
            renparam.width as u32, renparam.height as u32, render_web(&renparam)).unwrap());
        let mut buf: Vec<u8> = vec![];
        if let Ok(_) = imbuf.write_to(&mut buf, image::ImageOutputFormat::PNG) {
            // let enc = image::png::PNGEncoder::new();
            // let encresult = enc.encode(data, renparam.width, renparam.height, image::ColorType::Rgb8);//(output, &data, width as u32, height as u32, ColorType::RGB(8))
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Cache-Control", "no-cache")
                .body(Body::from(buf))
                .unwrap())
        }
        else{
            Ok(Response::new(Body::from("fail to render")))
        }
    }
    else{
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("empty"))
            .unwrap())
    }
}

async fn run_server(addr: SocketAddr) {
    println!("Listening on http://{}", addr);

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    // let make_svc = make_service_fn(|_conn| async {
    //     // service_fn converts our function into a `Service`
    //     Ok::<_, Infallible>(service_fn(hello_world))
    // });

    // Create a server bound on the provided address
    let serve_future = Server::bind(&addr)
        // Serve requests using our `async serve_req` function.
        // `serve` takes a closure which returns a type implementing the
        // `Service` trait. `service_fn` returns a value implementing the
        // `Service` trait, and accepts a closure which goes from request
        // to a future of the response. To use our `serve_req` function with
        // Hyper, we have to box it and put it in a compatability
        // wrapper to go from a futures 0.3 future (the kind returned by
        // `async fn`) to a futures 0.1 future (the kind used by Hyper).
        // .serve(|| service_fn(|req| serve_req(req, ren.clone()).boxed().compat()));
        .serve(make_service_fn(|_| async {
            Ok::<_, Error>(service_fn(|req| serve_req(req)))
        }));

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    if let Err(e) = serve_future.await {
        eprintln!("server error: {}", e);
    }
}

// static s_arc: RenderParams = RenderParams::new(
//     RenderParamStruct{
//         width: 1200,
//         height: 200,
//         ren: RenderEnv::new(
//             Vec3::new(0., -150., -300.), /* cam */
//             Vec3::new(0., -PI / 2., -PI / 2.), /* pyr */
//             1200,
//             200, /* xres, yres */
//             1.,
//             1., /* xfov, yfov*/
//             HashMap::new(),
//             vec![],
//             RenderColor::new(1.,1.,1.,1.), /* bgproc */
//         ).light(Vec3::new(50., 60., -50.))
//         .use_raymarching(use_raymarching)
//         .use_glow_effect(use_glow_effect, glow_effect),
//     }
// );

pub fn run_webserver(_ren: RenderParams) -> std::io::Result<()>{
    // Set the address to run our socket on.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Call our `run_server` function, which returns a future.
    // As with every `async fn`, for `run_server` to do anything,
    // the returned future needs to be run. Additionally,
    // we need to convert the returned future from a futures 0.3 future into a
    // futures 0.1 future.
    let futures_03_future = run_server(addr);
        // |req: Request<Body>| async {
        //     // let ren_clone = ren.clone();
        //     serve_req(req/*, ren_clone*/).await
        // });
    // let futures_01_future = futures_03_future.unit_error().boxed().compat();

    // Finally, we can run the future to completion using the `run` function
    // provided by Hyper.
    // run(futures_01_future);
    let mut rt = Runtime::new()?;
    rt.block_on(futures_03_future);

    Ok(())
}
