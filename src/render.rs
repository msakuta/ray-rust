
use crate::vec3::Vec3;
use vecmath;
use std::collections::HashMap;
use std::sync::Arc;

pub const MAX_REFLECTIONS: i32 = 3;
pub const MAX_REFRACTIONS: i32 = 10;


const OUTONLY: u32 = (1<<0);
const INONLY: u32 = (1<<1);
const RIGNORE: u32 = (1<<2);
const GIGNORE: u32 = (1<<3);
const BIGNORE: u32 = (1<<4);
// const RONLY: u32 = (GIGNORE|BIGNORE);
// const GONLY: u32 = (RIGNORE|BIGNORE);
// const BONLY: u32 = (RIGNORE|GIGNORE);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderColor{
    pub r: f32,
    pub g: f32,
    pub b: f32/*, _*/
}

impl RenderColor{
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self{r, g, b}
    }

    pub fn zero() -> Self {
        Self{r: 0., g: 0., b: 0.}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderPattern{
    Solid, Checkerboard, RepeatedGradation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMaterial{
    name: String,
    diffuse: RenderColor, /* Diffuse(R,G,B) */
    specular: RenderColor,/* Specular(R,G,B) */
    pn: i32,			/* Phong model index */
    t: f32, /* transparency, unit length per decay */
    n: f32, /* refraction constant */
    glow_dist: f32,
    frac: RenderColor, /* refraction per spectrum */
    pattern: RenderPattern,
    pattern_scale: f32,
}

trait RenderMaterialInterface{
    fn get_phong_number(&self) -> i32;
    fn get_transparency(&self) -> f32;
    fn get_refraction_index(&self) -> f32;
    fn lookup_texture(&self, pos: &Vec3) -> RenderColor;
}

impl RenderMaterial{
    pub fn new(
        name: String,
        diffuse: RenderColor,
        specular: RenderColor,
        pn: i32,
        t: f32,
        n: f32)
     -> RenderMaterial{
         RenderMaterial{
             name,
             diffuse,
             specular,
             pn,
             t,
             n,
             glow_dist: 0.,
             frac: RenderColor::new(1., 1., 1.),
             pattern: RenderPattern::Solid,
             pattern_scale: 1.,
         }
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> &str{
        &self.name
    }

    pub fn glow_dist(mut self, v: f32) -> Self{
        self.glow_dist = v;
        self
    }

    pub fn frac(mut self, frac: RenderColor) -> Self{
        self.frac = frac;
        self
    }

    pub fn pattern(mut self, pattern: RenderPattern) -> Self{
        self.pattern = pattern;
        self
    }

    pub fn pattern_scale(mut self, pattern_scale: f32) -> Self{
        self.pattern_scale = pattern_scale;
        self
    }
}

impl RenderMaterialInterface for RenderMaterial{
    fn get_phong_number(&self) -> i32{
        self.pn
    }

    fn get_transparency(&self) -> f32{
        self.t
    }

    fn get_refraction_index(&self) -> f32{
        self.n
    }

    fn lookup_texture(&self, pos: &Vec3) -> RenderColor{
        match self.pattern {
            RenderPattern::Solid => self.diffuse.clone(),
            RenderPattern::Checkerboard => {
                let ix = (pos.x / self.pattern_scale) as i32;
                let iy = (pos.z / self.pattern_scale) as i32;
                (if (ix + iy) % 2 == 0 {
                    RenderColor::new(0., 0., 0.)
                } else {
                    self.diffuse.clone()
                })
            }
            RenderPattern::RepeatedGradation => {
                RenderColor::new(
                    self.diffuse.r * (50. + (pos.x) / self.pattern_scale) % 1.,
                    self.diffuse.g * (50. + (pos.z) / self.pattern_scale) % 1.,
                    self.diffuse.b
                )
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderSphereSerial{
    material: String,
    r: f32,
    org: Vec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RenderFloorSerial{
    material: String,
    org: Vec3,		/* Center */
    face_normal: Vec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RenderObjectSerial{
    Sphere(RenderSphereSerial),
    Floor(RenderFloorSerial),
}

pub struct DeserializeError{
    pub s: String,
}

impl DeserializeError{
    fn new(s: &str) -> Self{
        DeserializeError{s: s.to_string()}
    }
}

impl Into<std::io::Error> for DeserializeError{
    fn into(self) -> std::io::Error{
        std::io::Error::new(std::io::ErrorKind::Other, "Deserialize error: ".to_string() + &self.s)
    }
}

impl From<serde_yaml::Error> for DeserializeError{
    fn from(_e: serde_yaml::Error) -> DeserializeError{
        DeserializeError{s: "serde_yaml::Error".to_string()}
    }
}

pub trait RenderObjectInterface{
    fn get_material(&self) -> &RenderMaterial;
    fn get_diffuse(&self, position: &Vec3) -> RenderColor;
    fn get_specular(&self, position: &Vec3) -> RenderColor;
    fn get_normal(&self, position: &Vec3) -> Vec3;
    fn raycast(&self, vi: &Vec3, eye: &Vec3, ray_length: f32, flags: u32) -> f32;
    fn distance(&self, vi: &Vec3) -> f32;
    fn serialize(&self) -> RenderObjectSerial;
}

pub struct RenderSphere{
    material: Arc<RenderMaterial>,
    r: f32,			/* Radius */
    org: Vec3,		/* Center */
}

impl RenderSphere{
    pub fn new(
        material: Arc<RenderMaterial>,
        r: f32,
        org: Vec3
    ) -> RenderObject {
        RenderObject::Sphere(RenderSphere{
            material,
            r,
            org,
        })
    }

    fn deserialize(ren: &RenderEnv, serial: &RenderSphereSerial) -> Result<RenderObject, DeserializeError>{
        Ok(Self::new(ren.materials.get(&serial.material)
            .ok_or(DeserializeError::new(&format!("RenderSphere couldn't find material {}", serial.material)))?
            .clone(),
            serial.r, serial.org))
    }
}

impl RenderObjectInterface for RenderSphere{
    fn get_material(&self) -> &RenderMaterial{
        &self.material
    }

    fn get_diffuse(&self, position: &Vec3) -> RenderColor{
        self.material.lookup_texture(position)
    }

    fn get_specular(&self, _position: &Vec3) -> RenderColor{
        self.material.specular.clone()
    }

    fn get_normal(&self, position: &Vec3) -> Vec3{
        (position - &self.org).normalized()
    }

    fn raycast(&self, vi: &Vec3, eye: &Vec3, ray_length: f32, flags: u32) -> f32{
        let obj = self;
        /* calculate vector from eye position to the object's center. */
        let wpt = vi - &obj.org;

        /* scalar product of the ray and the vector. */
        let b = 2.0f32 * eye.dot(&wpt);

        /* ??? */
        let c = wpt.dot(&wpt) - obj.r * obj.r;

        /* discriminant?? */
        let d2 = b * b - 4.0f32 * c;
        if d2 >= std::f32::EPSILON {
            let d = d2.sqrt();
            let t0 = (-b - d) as f32 / 2.0f32;
            if 0 == (flags & OUTONLY) && t0 >= 0.0f32 && t0 < ray_length {
                return t0;
            }
            else if 0 == (flags & INONLY) && 0f32 < (t0 + d) && t0 + d < ray_length {
                return t0 + d;
            }
        }

        ray_length
    }

    fn distance(&self, vi: &Vec3) -> f32{
        ((&self.org - vi).len() - self.r).max(0.)
    }

    fn serialize(&self) -> RenderObjectSerial{
        RenderObjectSerial::Sphere(RenderSphereSerial{
            material: self.material.name.clone(),
            org: self.org,
            r: self.r,
        })
    }
}

pub struct RenderFloor{
    material: Arc<RenderMaterial>,
    org: Vec3,		/* Center */
    face_normal: Vec3,
}

impl RenderFloor{
    pub fn new(
        material: Arc<RenderMaterial>,
        org: Vec3,
        face_normal: Vec3,
    ) -> RenderObject {
        RenderObject::Floor(RenderFloor{
            material,
            face_normal,
            org,
        })
    }

    fn deserialize(ren: &RenderEnv, serial: &RenderFloorSerial) -> Result<RenderObject, DeserializeError>{
        Ok(Self::new(ren.materials.get(&serial.material)
            .ok_or(DeserializeError::new(&format!("RenderFloor couldn't find material {}", serial.material)))?
            .clone(),
            serial.org, serial.face_normal))
    }
}

impl RenderObjectInterface for RenderFloor{
    fn get_material(&self) -> &RenderMaterial{
        &self.material
    }

    fn get_diffuse(&self, pt: &Vec3) -> RenderColor{
        self.material.lookup_texture(pt)
    }

    fn get_specular(&self, _position: &Vec3) -> RenderColor{
        self.material.specular.clone()
    }

    fn get_normal(&self, _: &Vec3) -> Vec3{
        self.face_normal
    }

    fn raycast(&self, vi: &Vec3, eye: &Vec3, ray_length: f32, _flags: u32) -> f32{
        let wpt = vi - &self.org;
        let w = self.face_normal.dot(eye);
        if /*fabs(w) > 1.0e-7*/ w <= 0. {
            let t0 = (-self.face_normal.dot(&wpt)) / w;
            if t0 >= 0. && t0 < ray_length {
                return t0;
            }
        }
        ray_length
    }

    fn distance(&self, vi: &Vec3) -> f32{
        (vi - &self.org).dot(&self.face_normal).max(0.)
    }

    fn serialize(&self) -> RenderObjectSerial{
        RenderObjectSerial::Floor(RenderFloorSerial{
            material: self.material.name.clone(),
            org: self.org,
            face_normal: self.face_normal,
        })
    }
}

pub enum RenderObject{
    Sphere(RenderSphere),
    Floor(RenderFloor)
}

impl RenderObject{
    pub fn get_interface(&self) -> &RenderObjectInterface{
        match self {
            RenderObject::Sphere(ref obj) => obj as &RenderObjectInterface,
            RenderObject::Floor(ref obj) => obj as &RenderObjectInterface,
        }
    }
}

pub struct RenderEnv{
    pub cam: Vec3, /* camera position */
    pub pyr: Vec3, /* camera direction in pitch yaw roll form */
    pub xres: i32,
    pub yres: i32,
    pub xfov: f32,
    pub yfov: f32,
    // Materials are stored in a string map, whose key is a string.
    // A material is stored in Arc in order to share between global material list
    // and each object. I'm not sure if it's better than embedding into each object.
    // We wanted to but cannot use reference (borrow checker gets mad about enums)
    // nor Rc (multithreading gets mad).
    pub materials: HashMap<String, Arc<RenderMaterial>>,
    pub objects: Vec<RenderObject>,
    pub light: Vec3,
    pub bgproc: fn(ren: &RenderEnv, pos: &Vec3) -> RenderColor,
    pub use_raymarching: bool,
    pub use_glow_effect: bool,
    glow_effect: f32,
}

#[derive(Serialize, Deserialize)]
struct Scene{
    materials: HashMap<String, RenderMaterial>,
    objects: Vec<RenderObjectSerial>,
}


impl RenderEnv{
    pub fn new(
        cam: Vec3, /* camera position */
        pyr: Vec3, /* camera direction in pitch yaw roll form */
        xres: i32,
        yres: i32,
        xfov: f32,
        yfov: f32,
        materials: HashMap<String, Arc<RenderMaterial>>,
        objects: Vec<RenderObject>,
        bgproc: fn(ren: &RenderEnv, pos: &Vec3) -> RenderColor
    ) -> Self{
        RenderEnv{
            cam, /* camera position */
            pyr, /* camera direction in pitch yaw roll form */
            xres,
            yres,
            xfov,
            yfov,
            materials,
            objects,
            light: Vec3::new(0., 0., 1.),
            bgproc,
            use_raymarching: false,
            use_glow_effect: false,
            glow_effect: 1.,
        }
    }

    pub fn light(mut self, light: Vec3) -> Self{
        self.light = light.normalized();
        self
    }

    pub fn use_raymarching(mut self, f: bool) -> Self{
        self.use_raymarching = f;
        self
    }

    pub fn use_glow_effect(mut self, f: bool, v: f32) -> Self{
        self.use_glow_effect = f;
        self.glow_effect = v;
        self
    }

    pub fn serialize(&self) -> Result<String, std::io::Error>{
        let mut sceneobj = Scene{
            materials: HashMap::new(),
            objects: self.objects.iter().map(|o| o.get_interface().serialize()).collect(),
        };
        for object in &self.objects {
            let material = object.get_interface().get_material();
            sceneobj.materials.insert(material.name.clone(), material.clone());
        }
        Ok(serde_yaml::to_string(&sceneobj)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?)
        // println!("{}", scene);
    }

    pub fn deserialize(&mut self, s: &str) -> Result<(), DeserializeError>{
        let sceneobj = serde_yaml::from_str::<Scene>(s)?;
        self.materials = sceneobj.materials.into_iter().map(|m| (m.0, Arc::new(m.1))).collect();
        self.objects.clear();
        for object in sceneobj.objects {
            match object {
                RenderObjectSerial::Sphere(ref sobj) =>
                    self.objects.push(RenderSphere::deserialize(self, sobj)?),
                RenderObjectSerial::Floor(ref sobj) =>
                    self.objects.push(RenderFloor::deserialize(self, sobj)?),
            }
        }
        Ok(())
    }
}


pub fn render(ren: &RenderEnv, pointproc: &mut FnMut(i32, i32, &RenderColor),
    thread_count: i32) {
    use vecmath::{Matrix3x4, row_mat3x4_mul};
    let mx: Matrix3x4<f32> = [
        [1., 0., 0., 0.],
        [0., ren.pyr.z.cos(), -ren.pyr.z.sin(), 0.],
        [0., ren.pyr.z.sin(), ren.pyr.z.cos(), 0.],
    ];

    let my = [
        [ren.pyr.y.cos(), -ren.pyr.y.sin(), 0., 0.],
        [ren.pyr.y.sin(), ren.pyr.y.cos(), 0., 0.],
        [0., 0., 1., 0.]
    ];

    let mp = [
        [ren.pyr.x.cos(), 0., ren.pyr.x.sin(), 0.],
        [0., 1., 0., 0.],
        [-ren.pyr.x.sin(), 0., ren.pyr.x.cos(), 0.],
    ];

    let view = row_mat3x4_mul(row_mat3x4_mul(mx, my), mp);

    // println!("Projection: {:?}", view);
/*	view.x[0][3] = 100.;
	view.x[1][3] = 0.;
	view.x[2][3] = 30.;*/

    let process_line = |iy: i32, point_middle: &mut FnMut(i32, i32, RenderColor)| {
        for ix in 0..ren.xres {
            let mut vi = ren.cam.clone();
            let mut eye: Vec3 = Vec3::new( /* cast ray direction vector? */
                1.,
                (ix - ren.xres / 2) as f32 * 2. * ren.xfov / ren.xres as f32,
                -(iy - ren.yres / 2) as f32 * 2. * ren.yfov / ren.yres as f32,
            );
            eye = Vec3::from(
                vecmath::row_mat3x4_transform_vec3(view, eye.into())).normalized();

            point_middle(ix, iy,
                if ren.use_raymarching { raymarch } else
                { raytrace }(ren, &mut vi, &mut eye, 0, None, 0) );
        }
    };

    if thread_count == 1 {
        let mut point_middle = |ix: i32, iy: i32, col: RenderColor| {
            pointproc(ix, iy, &col);
        };
        for iy in 0..ren.yres {
            process_line(iy, &mut point_middle);
        }
    }
    else{
        let scanlines = (ren.yres + thread_count - 1) / thread_count;
        println!("Splitting into {} scanlines; {} threads", scanlines, thread_count);
        crossbeam::scope(|scope| {
            // let handles: Vec<thread::JoinHandle<Vec<RenderColor>>> = (0..ren.yres).map(|iy| {
            let handles: Vec<crossbeam::thread::ScopedJoinHandle<'_, Vec<RenderColor>>> = (0..thread_count).map(|iy| {
                // println!("Started thread {}", iy);
                //let tx1 = mpsc::Sender::clone(&tx);
                scope.spawn(move |_| {
                    let mut linebuf = vec![RenderColor::zero(); (ren.xres * scanlines) as usize];
                    for iyy in 0..scanlines {
                        process_line(iy + iyy * thread_count, &mut |ix: i32, _iy: i32, col: RenderColor| {
                            linebuf[(ix + iyy * ren.xres) as usize] = col;
                        });
                    }

                    //tx1.send(i).unwrap();
                    // print!("Finished thread: {}\n", iy);
                    linebuf
                })
            }).collect();

            for (iy,h) in handles.into_iter().enumerate() {
                let results: Vec<RenderColor> = h.join().unwrap();
                for (ix,c) in results.iter().enumerate() {
                    let x = ix % ren.xres as usize;
                    let y = iy + ix / ren.xres as usize * thread_count as usize;
                    if y < ren.yres as usize {
                        pointproc(x as i32, y as i32, &c);
                    }
                }
                // print!("Joined thread: {}\n", iy);
            }
        }).expect("Worker thread join failed");
    }
}



/* find first object the ray hits */
/// @returns time at which ray intersects with a shape and its object id.
fn raycast(ren: &RenderEnv, vi: &Vec3, eye: &Vec3,
    ig: Option<&RenderObject>, flags: u32) -> (f32, usize)
{
    let mut t = std::f32::INFINITY;
    let mut ret_idx = 0;

	for (idx, obj) in ren.objects.iter().enumerate() {
        if let Some(ignore_obj) = ig {
            if ignore_obj as *const _ == obj as *const _ {
                continue;
            }
        }

        let obj_t = obj.get_interface().raycast(vi, eye, t, flags);
        if obj_t < t {
            t = obj_t;
            ret_idx = idx;
        }
    }

	(t, ret_idx)
}

fn shading(ren: &RenderEnv,
            idx: usize,
            n: &Vec3,
            pt: &Vec3,
            eye: &Vec3,
            nest: i32) -> RenderColor
{
    let o = &ren.objects[idx].get_interface();

    // let mut lv: f32;
    let (diffuse_intensity, reflected_ray, reflection_intensity) = {
        /* scalar product of light normal and surface normal */
        let light_incidence = ren.light.dot(n);
        let ln2 = 2.0 * light_incidence;
        let reflected_ray_to_light_source = &(n * ln2) - &ren.light;

        let eps = std::f32::EPSILON;
        let pn = o.get_material().get_phong_number();
        (
            light_incidence.max(0.),
            pt + &(&ren.light * eps),
            if 0 != pn {
                let reflection_incidence = -reflected_ray_to_light_source.dot(eye);
                if reflection_incidence > 0.0 { reflection_incidence.powi(pn) }
                else        { 0.0 }
            }
            else { 0. }
        )
    };

    /* shadow trace */
    let (k1, k2) = {
        let ray: Vec3 = ren.light.clone();
        let k1 = 0.2;
        if ren.use_raymarching {
            let RaymarchSingleResult{
                final_dist: _, idx, pos: _, iter, travel_dist, min_dist: _} = raymarch_single(ren, &reflected_ray, &ray, Some(&ren.objects[idx]));
            if FAR_AWAY <= travel_dist || MAX_ITER <= iter || 0. < ren.objects[idx].get_interface().get_material().get_transparency() {
                ((k1 + diffuse_intensity).min(1.), reflection_intensity)
            }
            else {
                (k1, 0.)
            }
        }
        else {
            let (t, i) = raycast(ren, &reflected_ray, &ray, Some(&ren.objects[idx]), 0);
            if t >= std::f32::INFINITY || 0. < ren.objects[i].get_interface().get_material().get_transparency() {
                ((k1 + diffuse_intensity).min(1.), reflection_intensity)
            }
            else {
                (k1, 0.)
            }
        }
    };

	/* face texturing */
		let kd = o.get_diffuse(pt);
	// else{
	// 	kd.fred = ren.objects[idx].kdr;
	// 	kd.fgreen = ren.objects[idx].kdg;
	// 	kd.fblue = ren.objects[idx].kdb;
	// }

	/* refraction! */
	if nest < MAX_REFRACTIONS && 0. < o.get_material().get_transparency() {
		let sp = eye.dot(&n);
		let f = o.get_material().get_transparency();

		let fc2 = {
            let frac = o.get_material().get_refraction_index();
			let reference = sp * (if sp > 0. { frac } else { 1. / frac } - 1.);
            let mut ray = (eye + &(n * reference)).normalized();
            let eps = std::f32::EPSILON;
			let mut pt3 = pt + &(&ray * eps);
            (if ren.use_raymarching { raymarch }
                else { raytrace })(ren, &mut pt3, &mut ray, nest,
                Some(&ren.objects[idx]), if sp < 0. { OUTONLY } else { INONLY })
		};
/*		t = raycast(ren, &reflectedRay, &ray, &i, &ren->objects[idx], OUTONLY);
		if(t < INFINITY)
		{
			Vec3 n2;
			f = exp(-t / o->t);
			normal(ren, i, &reflectedRay, &n2);
			shading(ren, i, &n2, &reflectedRay, &ray, &fc2, nest+1);
		}
		else{
			f = 0;
			ren->bgproc(&ray, &fc2);
		}*/
        RenderColor{
            r: (kd.r * k1 + k2) * (1. - f) + fc2.r * f,
            g: (kd.g * k1 + k2) * (1. - f) + fc2.g * f,
            b: (kd.b * k1 + k2) * (1. - f) + fc2.b * f,
        }
	}
	else{
		RenderColor{
            r: kd.r * k1 + k2,
            g: kd.g * k1 + k2,
            b: kd.b * k1 + k2,
        }
	}
}


fn raytrace(ren: &RenderEnv, vi: &mut Vec3, eye: &mut Vec3,
    mut lev: i32, init_ig: Option<&RenderObject>, mut flags: u32) -> RenderColor
{
    let mut fcs = RenderColor::new(1., 1., 1.);

	let mut ret_color = RenderColor::new(0., 0., 0.);
/*	bgcolor(eye, pColor);*/

    let mut ig: Option<&RenderObject> = init_ig;
	loop {
		lev += 1;
		let (t, idx) = raycast(ren, vi, eye, ig, flags);
		if t < std::f32::INFINITY {
/*			t -= EPS;*/

            /* shared point */
            // What a terrible formula... it's almost impractical to use it everywhere.
            let pt = &(&*eye * t) + vi;

            let o = &ren.objects[idx].get_interface();
            let n = o.get_normal(&pt);
            let face_color = shading(ren, idx,&n,&pt,eye, lev);
            // if idx == 2 {
            //     println!("Hit {}: eye: {:?} normal: {:?} shading: {:?}", idx, eye, n, face_color);
            // }

            let ks = o.get_specular(&pt);

            if 0 == (RIGNORE & flags) { ret_color.r += face_color.r * fcs.r; fcs.r *= ks.r; }
            if 0 == (GIGNORE & flags) { ret_color.g += face_color.g * fcs.g; fcs.g *= ks.g; }
            if 0 == (BIGNORE & flags) { ret_color.b += face_color.b * fcs.b; fcs.b *= ks.b; }
            if idx == 0 {
                break;
            }

			if (fcs.r + fcs.g + fcs.b) <= 0.1 {
				break;
            }

			if lev >= MAX_REFLECTIONS {
                break;
            }

			*vi = pt.clone();
			let en2 = -2.0 * eye.dot(&n);
			*eye += &n * en2;

			if n.dot(&eye) < 0. {
                flags &= !INONLY;
                flags |= OUTONLY;
            }
			else {
                flags &= !OUTONLY;
                flags |= INONLY;
            }

            ig = Some(&ren.objects[idx]);
        }
        else{
            let fc2 = (ren.bgproc)(ren, eye);
            ret_color.r	+= fc2.r * fcs.r;
            ret_color.g	+= fc2.g * fcs.g;
            ret_color.b	+= fc2.b * fcs.b;
        }
        if !(t < std::f32::INFINITY && lev < MAX_REFLECTIONS) {
            break;
        }
	}

    ret_color
}

fn distance_estimate(ren: &RenderEnv, vi: &Vec3,
    ig: Option<&RenderObject>) -> (f32, usize, f32)
{
    let mut closest_dist = std::f32::INFINITY;
    let mut ret_idx = 0;
    let mut glowing_dist = std::f32::INFINITY;

    for (idx, obj) in ren.objects.iter().enumerate() {
        if let Some(ignore_obj) = ig {
            if ignore_obj as *const _ == obj as *const _ {
                continue;
            }
        }

        let dist = obj.get_interface().distance(vi);
        if dist < closest_dist {
            closest_dist = dist;
            ret_idx = idx;
        }

        let glow = dist * obj.get_interface().get_material().glow_dist;
        if 0. < glow && glow < glowing_dist {
            glowing_dist = glow;
        }
    }

    (closest_dist, ret_idx, glowing_dist)
}

const RAYMARCH_EPS: f32 = 1e-3;
const FAR_AWAY: f32 = 1e4;
const MAX_ITER: usize = 10000;

struct RaymarchSingleResult{
    final_dist: f32,
    idx: usize,
    pos: Vec3,
    iter: usize,
    travel_dist: f32,
    min_dist: f32,
}

fn raymarch_single(ren: &RenderEnv, init_pos: &Vec3, eye: &Vec3, ig: Option<&RenderObject>)
    -> RaymarchSingleResult
{
    let mut iter = 0;
    let mut travel_dist = 0.;
    let mut pos = *init_pos;
    let mut min_dist = std::f32::INFINITY;
    loop {
        let (dist, idx, glowing_dist) = distance_estimate(ren, &pos, ig);
        pos = &(&*eye * dist) + &pos;
        travel_dist += dist;
        iter += 1;
        // let glowing_dist = ren.objects[idx].get_interface().get_glowing_dist();
        if glowing_dist < min_dist {
            min_dist = glowing_dist;
        }
        // println!("raymarch {:?} iter: {} pos: {:?}, dist: {}", eye, iter, pos, dist);
        if dist < RAYMARCH_EPS || FAR_AWAY < dist || MAX_ITER < iter {
            return RaymarchSingleResult{
                final_dist: dist,
                idx,
                pos,
                iter,
                travel_dist,
                min_dist,
            }
        }
    }
}

fn raymarch(ren: &RenderEnv, vi: &mut Vec3, eye: &mut Vec3,
    mut lev: i32, init_ig: Option<&RenderObject>, mut flags: u32) -> RenderColor
{
    // println!("using raymarch {:?}", eye);
    let mut fcs = RenderColor::new(1., 1., 1.);
    let mut pos = *vi;

    let mut ret_color = RenderColor::new(0., 0., 0.);
    let mut min_min_dist = std::f32::INFINITY;
/*	bgcolor(eye, pColor);*/

    let mut ig: Option<&RenderObject> = init_ig;
    loop {
        lev += 1;
        let RaymarchSingleResult{
            final_dist, idx, pos: pt, iter, travel_dist: _, min_dist} = raymarch_single(ren, &pos, eye, ig);
        if min_dist < min_min_dist {
            min_min_dist = min_dist;
        }
        if MAX_ITER < iter {
            // println!("Max iter reached: {:?} dist: {} idx: {}", eye, dist, idx);
        }
        if final_dist < RAYMARCH_EPS {
/*			t -= EPS;*/

            /* safe point */
            // What a terrible formula... it's almost impractical to use it everywhere.

            let o = &ren.objects[idx].get_interface();
            let n = o.get_normal(&pt);
            // let face_color = RenderColor::new(travel_dist / 100. % 1., 0., 0.);
            let face_color = shading(ren, idx,&n,&pt,eye, lev);
            // if idx == 2 {
                // println!("Hit {}: eye: {:?} normal: {:?} shading: {:?}", idx, eye, n, face_color);
            // }

            let ks = o.get_specular(&pt);

            if 0 == (RIGNORE & flags) { ret_color.r += face_color.r * fcs.r; fcs.r *= ks.r; }
            if 0 == (GIGNORE & flags) { ret_color.g += face_color.g * fcs.g; fcs.g *= ks.g; }
            if 0 == (BIGNORE & flags) { ret_color.b += face_color.b * fcs.b; fcs.b *= ks.b; }
            if idx == 0 {
                break;
            }

			if (fcs.r + fcs.g + fcs.b) <= 0.1 {
				break;
            }

			if lev >= MAX_REFLECTIONS {
                break;
            }

			pos = pt.clone();
			let en2 = -2.0 * eye.dot(&n);
			*eye += &n * en2;

			if n.dot(&eye) < 0. {
                flags &= !INONLY;
                flags |= OUTONLY;
            }
			else {
                flags &= !OUTONLY;
                flags |= INONLY;
            }

            ig = Some(&ren.objects[idx]);
        }
        else{
            let fc2 = (ren.bgproc)(ren, eye);
            ret_color.r	+= fc2.r * fcs.r;
            ret_color.g	+= fc2.g * fcs.g;
            ret_color.b	+= fc2.b * fcs.b;
        }
        if !(lev < MAX_REFLECTIONS) {
            break;
        }
	}
    // println!("raymarch loop end {:?}", eye);

    (if ren.use_glow_effect {
        let factor = if min_min_dist == std::f32::INFINITY { 1. }
            else { 1. + (0. + ren.glow_effect * (0.99f32).powf(min_min_dist)) };
        RenderColor::new(
            factor * ret_color.r,
            factor * ret_color.g,
            factor * ret_color.b)
    }
    else{
        ret_color
    })
}

