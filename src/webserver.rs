use crate::hyper_adapt::{make_payload_service, payload_service};
use crate::quat::Quat;
use crate::render::{render, RenderColor, RenderEnv};
use ::tokio::io::AsyncReadExt;
use ::tokio::runtime::Runtime;
use std::sync::Arc;
use std::thread;

use {
    hyper::{
        // Miscellaneous types from Hyper for working with HTTP.
        Body,
        Error,
        Request,
        Response,
        Server,
        StatusCode,
    },
    std::net::SocketAddr,
};

pub struct ServerParams {
    pub width: usize,
    pub height: usize,
    pub thread_count: i32,
    pub port_no: u16,
    pub ren: RenderEnv,
}

fn render_web(params: &ServerParams, ren: &RenderEnv) -> Vec<u8> {
    let (width, height) = (params.width, params.height);
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

    render(&ren, &mut putpoint, params.thread_count);
    data
}

async fn serve_req(
    req: Request<Body>,
    params: Arc<ServerParams>,
) -> Result<Response<Body>, hyper::Error> {
    println!(
        "Got request at {:?} in thread #{:?}",
        req.uri(),
        thread::current().id()
    );

    use std::f32::consts::PI;

    if req.uri() == "/" {
        Ok(Response::new(Body::from("<html>
        <head>
            <title>ray-rust</title>
            <script>
            window.onload = function(){
                var im = document.getElementById('render');
                var label = document.getElementById('label');
                ".to_string()
                + &format!("
                var x = {};
                var y = {};
                var z = {};
                var yaw = {};
                var pitch = {};\n",
                params.ren.camera.position.x,
                params.ren.camera.position.y,
                params.ren.camera.position.z,
                params.ren.camera.pyr.y * 180. / PI,
                params.ren.camera.pyr.x * 180. / PI)
                + "
                var buttonStates = {
                    w: false,
                    s: false,
                    a: false,
                    d: false,
                    q: false,
                    z: false,
                    ArrowRight: false,
                    ArrowLeft: false,
                    ArrowUp: false,
                    ArrowDown: false,
                };
                function updatePos(){
                    fetch(`/render?x=${x}&y=${y}&z=${z}&yaw=${yaw}&pitch=${pitch}`)
                        .then(function(response) {
                            if(response.ok) {
                                return response.blob();
                            }
                        })
                        .then(function(myBlob) { 
                            var objectURL = URL.createObjectURL(myBlob); 
                            im.src = objectURL;
                            tryUpdate();
                        }).catch(function(error) {
                            console.log('There has been a problem with your fetch operation: ', error.message);
                        });
                    label.innerHTML = `x=${x}<br>y=${y}<br>z=${z}<br>yaw=${yaw}<br>pitch=${pitch}`;
                }
                function tryUpdate(){
                    var ok = false;
                    if(buttonStates.a){
                        x += 10 * Math.sin(yaw * Math.PI / 180);
                        z += 10 * Math.cos(yaw * Math.PI / 180);
                        ok = true;
                    }
                    if(buttonStates.d){
                        x -= 10 * Math.sin(yaw * Math.PI / 180);
                        z -= 10 * Math.cos(yaw * Math.PI / 180);
                        ok = true;
                    }
                    if(buttonStates.w){
                        x += 10 * Math.cos(yaw * Math.PI / 180);
                        z -= 10 * Math.sin(yaw * Math.PI / 180);
                        ok = true;
                    }
                    if(buttonStates.s){
                        x -= 10 * Math.cos(yaw * Math.PI / 180);
                        z += 10 * Math.sin(yaw * Math.PI / 180);
                        ok = true;
                    }
                    if(buttonStates.q){
                        y += 10;
                        ok = true;
                    }
                    if(buttonStates.z){
                        y -= 10;
                        ok = true;
                    }
                    if(buttonStates.ArrowRight){
                        yaw += 5;
                        ok = true;
                    }
                    if(buttonStates.ArrowLeft){
                        yaw -= 5;
                        ok = true;
                    }
                    if(buttonStates.ArrowUp){
                        pitch -= 5;
                        ok = true;
                    }
                    if(buttonStates.ArrowDown){
                        pitch += 5;
                        ok = true;
                    }
                    if(ok){
                        updatePos();
                        return true;
                    }
                    return false;
                }
                updatePos();
                window.onkeydown = function(event){
                    if(event.key in buttonStates){
                        if(!buttonStates[event.key]){
                            console.log(`onkeydown x: ${x}, y: ${y}`)
                            buttonStates[event.key] = true;
                            tryUpdate();
                        }
                        event.preventDefault();
                    }
                }
                window.onkeyup = function(event){
                    if(event.key in buttonStates){
                        buttonStates[event.key] = false;
                        event.preventDefault();
                    }
                }
            }
            </script>
            <style>
                table { border-collapse: collapse; border: solid; }
            </style>
        </head>
        <body>
            <h1>ray-rust web interface</h1>
            <img id='render'>
            <hr>
            <h2>Controls</h2>
            <table border='1'>
            <tr><td>W</td><td>forward</td></tr>
            <tr><td>S</td><td>backward</td></tr>
            <tr><td>A</td><td>left</td></tr>
            <tr><td>D</td><td>right</td></tr>
            <tr><td>Q</td><td>up</td></tr>
            <tr><td>Z</td><td>down</td></tr>
            <tr><td>Left arrow</td><td>Turn left</td></tr>
            <tr><td>Right arrow</td><td>Turn right</td></tr>
            <tr><td>Up arrow</td><td>Turn up</td></tr>
            <tr><td>Down arrow</td><td>Turn down</td></tr>
            </table>
            <hr>
            <h2>Debug</h2>
            <div id='label'></div>
        </body>")))
    } else if req.uri() == "/image" {
        // render_web(&ren);
        if let Ok(mut image) = tokio::fs::File::open("barb.png").await {
            let mut buf: Vec<u8> = vec![];
            if image.read_to_end(&mut buf).await.is_ok() {
                println!("Responding with image {}", buf.len());
                Ok(Response::new(Body::from(buf)))
            } else {
                Ok(Response::new(Body::from("Error reading barb.png")))
            }
        } else {
            Ok(Response::new(Body::from("image")))
        }
    } else if req.uri().path() == "/render" {
        println!("GET /render, query = {:?}", req.uri().query());
        let (xpos, ypos, zpos, yaw, pitch) = if let Some(query) = req.uri().query() {
            let [mut xpos, mut ypos, mut zpos, mut yaw, mut pitch] = [0f32; 5];
            for s in query.split('&') {
                let tokens: Vec<_> = s.split('=').collect();
                match tokens[..] {
                    ["x", x] => {
                        if let Ok(f) = x.parse::<f32>() {
                            xpos = f;
                        }
                    }
                    ["z", z] => {
                        if let Ok(f) = z.parse::<f32>() {
                            zpos = f;
                        }
                    }
                    ["y", y] => {
                        if let Ok(f) = y.parse::<f32>() {
                            ypos = f;
                        }
                    }
                    ["yaw", ss] => {
                        if let Ok(f) = ss.parse::<f32>() {
                            yaw = f;
                        }
                    }
                    ["pitch", ss] => {
                        if let Ok(f) = ss.parse::<f32>() {
                            pitch = f;
                        }
                    }
                    _ => (),
                }
            }
            (xpos, ypos, zpos, yaw, pitch)
        } else {
            (0., 0., 0., 0., 0.)
        };
        println!(
            "Rendering with xpos={}, ypos={}, zpos={}, yaw={} pitch={}",
            xpos, ypos, zpos, yaw, pitch
        );

        // Cloning a whole RenderEnv object is dumb, but probably faster than deserializing from
        // a file in every request, and we need to modify camera position.
        let mut ren = params.ren.clone();
        ren.camera.position.x = xpos;
        ren.camera.position.y = ypos;
        ren.camera.position.z = zpos;
        ren.camera.pyr.y = yaw * PI / 180.;
        ren.camera.pyr.x = pitch * PI / 180.;
        ren.camera.rotation = Quat::from_pyr(&ren.camera.pyr);
        let imbuf = image::DynamicImage::ImageRgb8(
            image::ImageBuffer::from_raw(
                params.width as u32,
                params.height as u32,
                render_web(&params, &ren),
            )
            .unwrap(),
        );
        let mut buf: Vec<u8> = vec![];
        let mut cur = std::io::Cursor::new(&mut buf);
        if imbuf
            .write_to(&mut cur, image::ImageOutputFormat::Png)
            .is_ok()
        {
            // let enc = image::png::PNGEncoder::new();
            // let encresult = enc.encode(data, renparam.width, renparam.height, image::ColorType::Rgb8);//(output, &data, width as u32, height as u32, ColorType::RGB(8))
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Cache-Control", "no-cache")
                .header("Content-Type", "image/png")
                .body(Body::from(buf))
                .unwrap())
        } else {
            Ok(Response::new(Body::from("fail to render")))
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("empty"))
            .unwrap())
    }
}

async fn run_server(addr: SocketAddr, params: Arc<ServerParams>) {
    println!("Listening on http://{}", addr);

    // Create a server bound on the provided address
    let serve_future = Server::bind(&addr).serve(make_payload_service(
        |_, params| async move { Ok::<_, Error>(payload_service(serve_req, params)) },
        params,
    ));

    // Wait for the server to complete serving or exit with an error.
    // If an error occurred, print it to stderr.
    if let Err(e) = serve_future.await {
        eprintln!("server error: {}", e);
    }
}

pub fn run_webserver(params: Arc<ServerParams>) -> std::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], params.as_ref().port_no));

    let future = run_server(addr, params);

    let rt = Runtime::new()?;
    rt.block_on(future);

    Ok(())
}
