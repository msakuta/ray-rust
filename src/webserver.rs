use crate::render::{render, RenderEnv, RenderColor};
use std::sync::Arc;

use {
    hyper::{
        // Miscellaneous types from Hyper for working with HTTP.
        Body, Client, Request, Response, Server, Uri, StatusCode,
        Error,

        // This function turns a closure which returns a future into an
        // implementation of the the Hyper `Service` trait, which is an
        // asynchronous function from a generic `Request` to a `Response`.
        service::service_fn,
        service::make_service_fn,

        // A function which runs a future to completion using the Hyper runtime.
        // rt::run,
    },
    futures::{
        // Extension trait for futures 0.1 futures, adding the `.compat()` method
        // which allows us to use `.await` on 0.1 futures.
        compat::Future01CompatExt,
        // Extension traits providing additional methods on futures.
        // `FutureExt` adds methods that work for all futures, whereas
        // `TryFutureExt` adds methods to futures that return `Result` types.
        future::{FutureExt, TryFutureExt},
    },
    std::net::SocketAddr,
};

pub struct RenderParamStruct{
    pub width: usize,
    pub height: usize,
    pub thread_count: i32,
    pub ren: RenderEnv,
}

pub type RenderParams = Arc<RenderParamStruct>;

fn render_web(ren: &RenderParams){
    let (width, height) = (ren.width, ren.height);
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

    render(&ren.ren, &mut putpoint, ren.thread_count);
}

async fn serve_req(req: Request<Body>, ren: RenderParams) -> Result<Response<Body>, hyper::Error> {
    // Always return successfully with a response containing a body with
    // a friendly greeting ;)
    println!("Got request at {:?}", req.uri());
    if req.uri() == "/" {
        Ok(Response::new(Body::from("<html>
        <head>
            <title>ray-rust</title>
        </head>
        <body>
            <h1>hello, world!</h1>
            <img src='/image'>
        </body>")))
    }
    else if req.uri() == "/image" {
        render_web(&ren);
        Ok(Response::new(Body::from("empty")))
    }
    else{
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("empty"))
            .unwrap())
    }
}

async fn run_server(addr: SocketAddr, ren: RenderParams) {
    println!("Listening on http://{}", addr);
    let ren_clone = ren.clone();

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
            Ok::<_, Error>(service_fn(|_req| async {
                Ok::<_, Error>(Response::new(Body::from("Hello World")))
            }))
        }));

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    // if let Err(e) = serve_future.compat().await {
    //     eprintln!("server error: {}", e);
    // }
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

pub fn run_webserver(ren: RenderParams) -> std::io::Result<()>{
    // Set the address to run our socket on.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // Call our `run_server` function, which returns a future.
    // As with every `async fn`, for `run_server` to do anything,
    // the returned future needs to be run. Additionally,
    // we need to convert the returned future from a futures 0.3 future into a
    // futures 0.1 future.
    let futures_03_future = run_server(addr, ren);
    let futures_01_future = futures_03_future.unit_error().boxed().compat();

    // Finally, we can run the future to completion using the `run` function
    // provided by Hyper.
    // run(futures_01_future);

    Ok(())
}
