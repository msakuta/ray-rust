
use crate::vec3::{Vec3, Mat4, MAT4IDENTITY, matcat, concat};

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

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct RenderMaterial{
	diffuse: RenderColor, /* Diffuse(R,G,B) */
	specular: RenderColor,/* Specular(R,G,B) */
	pn: i32,			/* Phong model index */
	t: f32, /* transparency, unit length per decay */
	n: f32, /* refraction constant */
	frac: RenderColor /* refraction per spectrum */
}

trait RenderMaterialInterface{
    fn get_phong_number(&self) -> i32;
    fn get_transparency(&self) -> f32;
    fn get_refraction_index(&self) -> f32;
}

impl RenderMaterial{
    pub fn new(
        diffuse: RenderColor,
        specular: RenderColor,
        pn: i32,
        t: f32,
        n: f32)
     -> RenderMaterial{
         RenderMaterial{
             diffuse,
             specular,
             pn,
             t,
             n,
             frac: RenderColor::new(1., 1., 1.),
         }
    }

    pub fn frac(mut self, frac: RenderColor) -> Self{
        self.frac = frac;
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
}

trait RenderObjectInterface{
    fn get_material(&self) -> &RenderMaterial;
    fn get_diffuse(&self, position: &Vec3) -> RenderColor;
    fn get_specular(&self, position: &Vec3) -> RenderColor;
    fn get_normal(&self, position: &Vec3) -> Vec3;
    fn raycast(&self, vi: &Vec3, eye: &Vec3, ray_length: f32, flags: u32) -> f32;
}

pub struct RenderSphere{
    material: RenderMaterial,
    r: f32,			/* Radius */
    org: Vec3,		/* Center */
}

impl RenderSphere{
    pub fn new(
        material: RenderMaterial,
        r: f32,
        org: Vec3
    ) -> RenderObject {
        RenderObject::Sphere(RenderSphere{
            material,
            r,
            org,
        })
    }
}

impl RenderObjectInterface for RenderSphere{
    fn get_material(&self) -> &RenderMaterial{
        &self.material
    }

    fn get_diffuse(&self, _position: &Vec3) -> RenderColor{
        self.material.diffuse.clone()
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
}

pub struct RenderFloor{
    material: RenderMaterial,
    org: Vec3,		/* Center */
    face_normal: Vec3,
}

impl RenderFloor{
    pub fn new(
        material: RenderMaterial,
        org: Vec3,
        face_normal: Vec3,
    ) -> RenderObject {
        RenderObject::Floor(RenderFloor{
            material,
            face_normal,
            org,
        })
    }
}

impl RenderObjectInterface for RenderFloor{
    fn get_material(&self) -> &RenderMaterial{
        &self.material
    }

    fn get_diffuse(&self, pt: &Vec3) -> RenderColor{
        RenderColor::new(
            self.material.diffuse.r * (50. + (pt.x - self.org.x) / 300.) % 1.,
            self.material.diffuse.g * (50. + (pt.z - self.org.z) / 300.) % 1.,
            self.material.diffuse.b
        )
/*	{
	double d, dd;
	d = fmod(50. + (pt->x - ths->org.x) / 300., 1.) - .5;
	dd = fmod(50. + (pt->z - ths->org.z) / 300., 1.) - .5;
	kd->r = kd->g = kd->b = .5 / (250. * (d * d * dd * dd) + 1.);
	}*/
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
}

pub enum RenderObject{
    Sphere(RenderSphere),
    Floor(RenderFloor)
}

impl RenderObject{
    fn get_interface(&self) -> &RenderObjectInterface{
        match self {
            RenderObject::Sphere(obj) => obj as &RenderObjectInterface,
            RenderObject::Floor(obj) => obj as &RenderObjectInterface,
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
    pub objects: Vec<RenderObject>,
    pub nobj: usize,
    pub light: Vec3,
    pub bgproc: fn(ren: &RenderEnv, pos: &Vec3) -> RenderColor
}

impl RenderEnv{
    pub fn new(
        cam: Vec3, /* camera position */
        pyr: Vec3, /* camera direction in pitch yaw roll form */
        xres: i32,
        yres: i32,
        xfov: f32,
        yfov: f32,
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
            objects,
            nobj: objects.len(),
            light: Vec3::new(0., 0., 1.),
            bgproc,
        }
    }

    pub fn light(mut self, light: Vec3) -> Self{
        self.light = light.normalized();
        self
    }
}


pub fn render(ren: &RenderEnv, pointproc: &mut FnMut(i32, i32, &RenderColor),
    use_multithread: bool) {
	let mut view: Mat4 = MAT4IDENTITY;

	{
		let mr = [
			[1., 0., 0., 0.],
			[0., ren.pyr.z.cos(), -ren.pyr.z.sin(), 0.],
			[0., ren.pyr.z.sin(), ren.pyr.z.cos(), 0.],
        ];
		view = matcat(&view, &mr);
	}
	{
		let my = [
			[ren.pyr.y.cos(), -ren.pyr.y.sin(), 0., 0.],
			[ren.pyr.y.sin(), ren.pyr.y.cos(), 0., 0.],
			[0., 0., 1., 0.]
        ];
		view = matcat(&view, &my);
	}
	{
		let mp = [
			[ren.pyr.x.cos(), 0., ren.pyr.x.sin(), 0.],
			[0., 1., 0., 0.],
			[-ren.pyr.x.sin(), 0., ren.pyr.x.cos(), 0.],
        ];
		view = matcat(&view, &mp);
	}
    // println!("Projection: {:?}", view);
/*	view.x[0][3] = 100.;
	view.x[1][3] = 0.;
	view.x[2][3] = 30.;*/

    let process_line = |iy: i32, point_middle: &mut FnMut(i32, i32, &RenderColor)| {
        for ix in 0..ren.xres {
            let mut vi = ren.cam.clone();
            let mut eye: Vec3 = Vec3::new( /* cast ray direction vector? */
                1.,
                (ix - ren.xres / 2) as f32 * 2. * ren.xfov / ren.xres as f32,
                -(iy - ren.yres / 2) as f32 * 2. * ren.yfov / ren.yres as f32,
            );
            eye = concat(&view, &eye).normalized();

            point_middle(ix, iy, &raytrace(ren, &mut vi, &mut eye, 0, 0));
        }
    };

    if use_multithread {
        let mut point_middle = |ix: i32, iy: i32, col: &RenderColor| {
            pointproc(ix, iy, col);
        };
        for iy in 0..ren.yres {
            process_line(iy, &mut point_middle);
        }
    }
    else{
        use std::thread;
        let handles: Vec<thread::JoinHandle<Vec<RenderColor>>> = (0..ren.yres).map(|iy| {
            let xres = ren.xres;
            println!("Started thread {}", iy);
            //let tx1 = mpsc::Sender::clone(&tx);
            thread::spawn(||{
                let mut linebuf = vec![RenderColor::zero(); xres as usize];
                process_line(iy, &mut |ix: i32, _iy: i32, col: &RenderColor| {
                    linebuf[ix as usize] = col.clone();
                });

                //tx1.send(i).unwrap();
                print!("Finished thread: {}\n", iy);
                linebuf
            })
        }).collect();

        for (iy,h) in handles.into_iter().enumerate() {
            let results: Vec<RenderColor> = h.join().unwrap();
            for (ix,c) in results.iter().enumerate() {
                pointproc(ix as i32, iy as i32, &c);
            }
            print!("Joined thread: {}\n", iy);
        }
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
        let (t, i) = raycast(ren, &reflected_ray, &ray, Some(&ren.objects[idx]), 0);
        if t >= std::f32::INFINITY || 0. < ren.objects[i].get_interface().get_material().get_transparency() {
            ((k1 + diffuse_intensity).min(1.), reflection_intensity)
        }
        else {
            (k1, 0.)
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
            raytrace(ren, &mut pt3, &mut ray, nest, if sp < 0. { OUTONLY } else { INONLY })
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
    mut lev: i32, mut flags: u32) -> RenderColor
{
    let mut fcs = RenderColor::new(1., 1., 1.);

	let mut ret_color = RenderColor::new(0., 0., 0.);
/*	bgcolor(eye, pColor);*/

    let mut ig: Option<&RenderObject> = None;
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

