


pub const MAXLEVEL: i32 = 1;
pub const MAXREFLAC: i32 = 10;


const OUTONLY: u32 = (1<<0);
const INONLY: u32 = (1<<1);
// const RIGNORE: u32 = (1<<2);
// const GIGNORE: u32 = (1<<3);
// const BIGNORE: u32 = (1<<4);
// const RONLY: u32 = (GIGNORE|BIGNORE);
// const GONLY: u32 = (RIGNORE|BIGNORE);
// const BONLY: u32 = (RIGNORE|GIGNORE);

#[derive(Debug, Clone)]
pub struct FCOLOR{
	pub fred: f32,
	pub fgreen: f32,
	pub fblue: f32,
//	reserved: f32;
}

impl FCOLOR{
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self{fred: r, fgreen: g, fblue: b}
    }
}

#[derive(Debug, Clone)]
pub struct RenderColor{
	r: f32,
    g: f32,
    b: f32/*, _*/
}

impl RenderColor{
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self{r, g, b}
    }
}

#[derive(Clone, Debug)]
pub struct Vec3{
	x: f32,
	y: f32,
	z: f32,
	reserved: f32,
}

impl Vec3{
    pub fn new(x: f32, y: f32, z: f32) -> Vec3{
        Vec3{x, y, z, reserved: 1.}
    }

    pub fn zero() -> Self{
        Self::new(0., 0., 0.)
    }

    pub fn dot(&self, b: &Self) -> f32 {
        self.x*b.x
         + self.y*b.y
         + self.z*b.z
    } 

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let len = self.len();
        Vec3::new(self.x / len, self.y / len, self.z / len)
    }

    pub fn normalize(&mut self){
        let len = self.len();
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }
}

fn floorkd(ths: &SOBJECT, pt: &Vec3) -> RenderColor{
    RenderColor::new(
        (50. + (pt.x - ths.org.x) / 300.) % 1.,
        (50. + (pt.z - ths.org.z) / 300.) % 1.,
        1.
    )
/*	{
	double d, dd;
	d = fmod(50. + (pt->x - ths->org.x) / 300., 1.) - .5;
	dd = fmod(50. + (pt->z - ths->org.z) / 300., 1.) - .5;
	kd->r = kd->g = kd->b = .5 / (250. * (d * d * dd * dd) + 1.);
	}*/
}

#[allow(dead_code)]
pub struct RenderObjectStatic{
	kdproc: fn(ths: &RenderObject, pt: &Vec3) -> RenderColor,
	ksproc: fn(ths: &RenderObject, pt: &Vec3) -> RenderColor,
}

#[allow(non_upper_case_globals)]
pub const floor_static: RenderObjectStatic = RenderObjectStatic{
    kdproc: floorkd,
    ksproc: floorkd,
};

fn kdproc_def(ths: &RenderObject, _: &Vec3) -> RenderColor{
	ths.diffuse.clone()
}
fn ksproc_def(ths: &RenderObject, _: &Vec3) -> RenderColor{
	ths.specular.clone()
}

#[allow(non_upper_case_globals)]
pub const render_object_static_def: RenderObjectStatic = RenderObjectStatic{
    kdproc: kdproc_def,
    ksproc: ksproc_def,
};

#[allow(dead_code)]
pub struct RenderObject{
	vft: &'static RenderObjectStatic, /* virtual function table */
	r: f32,			/* Radius */
	org: Vec3,		/* Center */
	diffuse: RenderColor, /* Diffuse(R,G,B) */
	specular: RenderColor,/* Specular(R,G,B) */
	pn: i32,			/* Phong model index */
	t: f32, /* transparency, unit length per decay */
	n: f32, /* reflaction constant */
	frac: RenderColor /* reflaction per spectrum */
}

pub type SOBJECT = RenderObject;

impl RenderObject{
    pub fn new(
        vft: &'static RenderObjectStatic,
        r: f32, org: Vec3,
        diffuse: RenderColor,
        specular: RenderColor,
        pn: i32, t: f32, n: f32,
        frac: RenderColor
    ) -> RenderObject {
        RenderObject{
            vft,
            r,
            org,
            diffuse,
            specular,
            pn,
            t,
            n,
            frac,
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
    pub objects: Vec<SOBJECT>,
    pub nobj: i32,
    pub light: Vec3,
    pub vnm: Vec3,
    pub bgproc: fn(pos: &Vec3, fcolor: &mut RenderColor)
}

type Mat4 = [[f32; 4]; 3];

const MAT4IDENTITY: Mat4 = [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.]];

fn concat(m: &Mat4, v: &Vec3) -> Vec3{
    Vec3{
        x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3],
        y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3],
        z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3],
        reserved: 0.
    }
}


fn matcat(m1: &Mat4, m2: &Mat4) -> Mat4{
	let mut ret: Mat4 = MAT4IDENTITY;
	for i in 0..3 {
        for j in 0..4 {
            ret[i][j] =
                m1[i][0] * m2[0][j] +
                m1[i][1] * m2[1][j] +
                m1[i][2] * m2[2][j];
        }
    }
    ret
}



pub fn render(ren: &mut RenderEnv, pointproc: &mut FnMut(i32, i32, &FCOLOR)) {
	let mut view: Mat4 = MAT4IDENTITY;

	ren.light.normalize();
	ren.vnm.normalize();

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

    for iy in 0..ren.yres {
        for ix in 0..ren.xres {
            let mut vi = ren.cam.clone();
            let mut eye: Vec3 = Vec3{ /* cast ray direction vector? */
                x: 1.,
                y: (ix - ren.xres / 2) as f32 * 2. * ren.xfov / ren.xres as f32,
                z: -(iy - ren.yres / 2) as f32 * 2. * ren.yfov / ren.yres as f32,
                reserved: 0.
            };
            eye = concat(&view, &eye).normalized();

            pointproc(ix, iy, &raytrace(ren, &mut vi, &mut eye, 0, 0));
        }
    }
}


/* find first object the ray hits */
/// @returns time at which ray intersects with a shape and its object id.
fn raycast(ren: &RenderEnv, vi: &Vec3, eye: &Vec3,
    ig: Option<&SOBJECT>, flags: u32) -> (f32, usize)
{
    let mut t = std::f32::INFINITY;
    let mut ret_idx = 0;

	for (idx, obj) in ren.objects.iter().enumerate() {
        if let Some(ignore_obj) = ig {
            if ignore_obj as *const _ == obj as *const _ {
                continue;
            }
        }
        /* calculate vector from eye position to the object's center. */
        let wpt = Vec3::new(
            vi.x - obj.org.x,
            vi.y - obj.org.y,
            vi.z - obj.org.z
        );

        /* scalar product of the ray and the vector. */
        let b = 2.0f32 * (eye.x * wpt.x + eye.y * wpt.y + eye.z * wpt.z);

        /* ??? */
        let c = wpt.x * wpt.x + wpt.y * wpt.y + wpt.z * wpt.z -
                obj.r * obj.r;

        /* discriminant?? */
        let d2 = b * b - 4.0f32 * c;
        if d2 >= std::f32::EPSILON {
            let d = d2.sqrt();
            let t0 = (-b - d) as f32 / 2.0f32;
            if 0 == (flags & OUTONLY) && t0 >= 0.0f32 && t0 < t {
                t = t0;
                ret_idx = idx;
            }
            else if 0 == (flags & INONLY) && 0f32 < (t0 + d) && t0 + d < t {
                t = t0 + d;
                ret_idx = idx;
            }
        }
    }

    let wpt = Vec3::new(
        vi.x - ren.objects[0].org.x,
        vi.y - ren.objects[0].org.y,
        vi.z - ren.objects[0].org.z
    );
	let w = ren.vnm.x * eye.x + ren.vnm.y * eye.y + ren.vnm.z * eye.z;
	if /*fabs(w) > 1.0e-7*/ w <= 0. {
		let t0 = (-ren.vnm.x * wpt.x - ren.vnm.y * wpt.y - ren.vnm.z * wpt.z) / w;
		if t0 >= 0. && t0 < t {
			t = t0;
			ret_idx = 0;
		}
	}

	(t, ret_idx)
}

/* calculate normalized normal vector */
fn normal_vector(ren: &RenderEnv, idx: usize, pt: &Vec3) -> Vec3
{
    if 0 == idx { ren.vnm.clone() }
    else{
        Vec3::new(
            pt.x - ren.objects[idx].org.x,
            pt.y - ren.objects[idx].org.y,
            pt.z - ren.objects[idx].org.z
        ).normalized()
	}
}

fn shading(ren: &mut RenderEnv,
            idx: usize,
            n: &Vec3,
            pt: &Vec3,
            eye: &Vec3,
            nest: i32) -> FCOLOR
{
    // let mut lv: f32;
    let (diffuse_intensity, reflected_ray, reflection_intensity) = {
        let o = &ren.objects[idx];

        /* scalar product of light normal and surface normal */
        let light_incidence = ren.light.dot(n);
        let ln2 = 2.0 * light_incidence;
        let reflected_ray_to_light_souce = Vec3::new(
            ln2 * n.x - ren.light.x,
            ln2 * n.y - ren.light.y,
            ln2 * n.z - ren.light.z,
        );

        let eps = std::f32::EPSILON;
        (
            light_incidence.max(0.),
            Vec3::new(
                pt.x + ren.light.x * eps,
                pt.y + ren.light.y * eps,
                pt.z + ren.light.z * eps,
            ),
            if 0 != o.pn {
                let reflection_incidence = -reflected_ray_to_light_souce.dot(eye);
                if reflection_incidence > 0.0 { reflection_incidence.powi(o.pn) }
                else        { 0.0 }
            }
            else { 0. }
        )
    };

    /* shadow trace */
    let (k1, k2) = {
        let ray: Vec3 = ren.light.clone();
        let k1 = 0.2;
        let (t, _) = raycast(ren, &reflected_ray, &ray, Some(&ren.objects[idx]), 0);
        if t >= std::f32::INFINITY || 0. < ren.objects[idx].t {
            (k1 + diffuse_intensity, reflection_intensity)
        }
        else {
            (k1, 0.)
        }
    };

    let o = &ren.objects[idx];
	/* face texturing */
		let kd = (o.vft.kdproc)(o, pt);
	// else{
	// 	kd.fred = ren.objects[idx].kdr;
	// 	kd.fgreen = ren.objects[idx].kdg;
	// 	kd.fblue = ren.objects[idx].kdb;
	// }

	/* refraction! */
	if nest < MAXREFLAC && 0. < ren.objects[idx].t {
		let sp = eye.dot(&n);
		let f = o.t;

		let fc2 = {
			let reference = sp * (if sp > 0. { ren.objects[idx].n } else { 1. / ren.objects[idx].n } - 1.);
            let mut ray = Vec3::new(
                eye.x + reference * n.x,
                eye.y + reference * n.y,
                eye.z + reference * n.z
            ).normalized();
            let eps = std::f32::EPSILON;
			let mut pt3 = Vec3::new(
                pt.x + ray.x * eps,
                pt.y + ray.y * eps,
                pt.z + ray.z * eps,
            );
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
        FCOLOR{
            fred: (kd.r * k1 + k2) * (1. - f) + fc2.fred * f,
            fgreen: (kd.g * k1 + k2) * (1. - f) + fc2.fgreen * f,
            fblue: (kd.b * k1 + k2) * (1. - f) + fc2.fblue * f,
        }
	}
	else{
		FCOLOR{
            fred: kd.r * k1 + k2,
            fgreen: kd.g * k1 + k2,
            fblue: kd.b * k1 + k2,
        }
	}
}


fn raytrace(ren: &mut RenderEnv, vi: &mut Vec3, eye: &mut Vec3,
    mut lev: i32, mut flags: u32) -> FCOLOR
{
    let fcs = FCOLOR::new(1., 1., 1.);

	let mut ret_color = FCOLOR::new(0., 0., 0.);
/*	bgcolor(eye, pColor);*/

    let mut ig: Option<&SOBJECT> = None;
	loop {
		lev += 1;
		let (t, idx) = raycast(ren, vi, eye, ig, flags);
		if t < std::f32::INFINITY {
/*			t -= EPS;*/

			/* shared point */
            let mut pt = Vec3::zero();
            pt.x = eye.x * t + vi.x;
            pt.y = eye.y * t + vi.y;
            pt.z = eye.z * t + vi.z;

			let n = normal_vector(ren, idx, &pt);
			let fc = shading(ren, idx,&n,&pt,eye, lev);
            // if idx == 2 {
            //     println!("Hit {}: eye: {:?} normal: {:?} shading: {:?}", idx, eye, n, fc);
            // }

			// let o: &SOBJECT = &ren.objects[idx];
			// let ks = (o.vft.ksproc)(o, &pt);
			// else{
			// 	ks.r = o.ksr;
			// 	ks.g = o.ksg;
			// 	ks.b = o.ksb;
			// }

			// if(0 != (RIGNORE & flags)) { pColor.fred	+= fc.fred * fcs.fred; fcs.fred	*= ks.r; }
			// if(0 != (GIGNORE & flags)) { pColor.fgreen	+= fc.fgreen * fcs.fgreen; fcs.fgreen	*= ks.g; }
			// if(0 != (BIGNORE & flags)) { pColor.fblue	+= fc.fblue * fcs.fblue; fcs.fblue	*= ks.b; }
            ret_color = fc.clone();

			if (fcs.fred + fcs.fgreen + fcs.fblue) <= 0.1 {
				break;
            }

			if lev >= MAXLEVEL {
                break;
            }

			*vi = pt.clone();
			let en2 = 2.0 * (-eye.x * n.x - eye.y * n.y - eye.z * n.z);
			eye.x += en2 * n.x; eye.y += en2 * n.y; eye.z += en2 * n.z;

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
			let mut fc2 = RenderColor::new(0., 0., 0.);
			(ren.bgproc)(eye, &mut fc2);
			ret_color.fred	+= fc2.r * fcs.fred;
			ret_color.fgreen	+= fc2.g * fcs.fgreen;
			ret_color.fblue	+= fc2.b * fcs.fblue;
		}
        if !(t < std::f32::INFINITY && lev < MAXLEVEL) {
            break;
        }
	}

    ret_color
}

