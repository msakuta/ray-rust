

const OUTONLY: u32 = (1<<0);
const INONLY: u32 = (1<<1);

pub struct FCOLOR{
	pub fred: f32,
	pub fgreen: f32,
	pub fblue: f32,
//	reserved: f32;
}

pub struct render_fcolor{
	r: f32,
    g: f32,
    b: f32/*, _*/
}

pub type fcolor_t = render_fcolor;

impl fcolor_t{
    pub fn new(r: f32, g: f32, b: f32) -> fcolor_t {
        fcolor_t{r, g, b}
    }
}

pub struct POS3D{
	x: f32,
	y: f32,
	z: f32,
	reserved: f32,
}

impl POS3D{
    pub fn new(x: f32, y: f32, z: f32) -> POS3D{
        POS3D{x, y, z, reserved: 1.}
    }
}

fn floorkd(ths: &SOBJECT, pt: &POS3D, kd: &mut fcolor_t){
	{
		kd.r = (50. + (pt.x - ths.org.x) / 300.) % 1.;
		kd.g = (50. + (pt.z - ths.org.z) / 300.) % 1.;
	}
/*	{
	double d, dd;
	d = fmod(50. + (pt->x - ths->org.x) / 300., 1.) - .5;
	dd = fmod(50. + (pt->z - ths->org.z) / 300., 1.) - .5;
	kd->r = kd->g = kd->b = .5 / (250. * (d * d * dd * dd) + 1.);
	}*/
}

pub struct render_object_static{
	kdproc: fn(ths: &render_object, pt: &POS3D, kd: &mut fcolor_t),
	ksproc: fn(ths: &render_object, pt: &POS3D, ks: &mut fcolor_t),
}

pub type SOBJECT_S = render_object_static;

pub const floor_static: SOBJECT_S = SOBJECT_S{
    kdproc: floorkd,
    ksproc: floorkd,
};

fn kdproc_def(ths: &render_object, pt: &POS3D, kd: &mut fcolor_t){
	kd.r = ths.kdr; kd.g = ths.kdg; kd.b = ths.kdb;
}
fn ksproc_def(ths: &render_object, pt: &POS3D, ks: &mut fcolor_t){
	ks.r = ths.ksr; ks.g = ths.ksg; ks.b = ths.ksb;
}

pub const render_object_static_def: SOBJECT_S = SOBJECT_S{
    kdproc: kdproc_def,
    ksproc: ksproc_def,
};


pub struct render_object{
	vft: &'static SOBJECT_S, /* virtual function table */
	r: f32,			/* Radius */
	org: POS3D,		/* Center */
	kdr: f32, kdg: f32, kdb: f32, /* Diffuse(R,G,B) */
	ksr: f32, ksg: f32, ksb: f32,/* Specular(R,G,B) */
	pn: i32,			/* Phong model index */
	t: f32, /* transparency, unit length per decay */
	n: f32, /* reflaction constant */
	frac: fcolor_t /* reflaction per spectrum */
}

pub type SOBJECT = render_object;

impl render_object{
    pub fn new(
        vft: &'static SOBJECT_S,
        r: f32, org: POS3D,
        kdr: f32, kdg: f32, kdb: f32,
        ksr: f32, ksg: f32, ksb: f32,
        pn: i32, t: f32, n: f32,
        frac: fcolor_t
    ) -> render_object {
        render_object{
            vft: &render_object_static_def,
            r,
            org,
            kdr, kdg, kdb,
            ksr, ksg, ksb,
            pn,
            t,
            n,
            frac,
        }
    }
}

pub struct render_env{
    pub cam: POS3D, /* camera position */
    pub pyr: POS3D, /* camera direction in pitch yaw roll form */
    pub xres: i32,
    pub yres: i32,
    pub xfov: f32,
    pub yfov: f32,
    pub objects: Vec<SOBJECT>,
    pub nobj: i32,
    pub light: POS3D,
    pub vnm: POS3D,
    pub bgproc: fn(pos: &POS3D, fcolor: &mut fcolor_t)
}

type RENDER = render_env;

type mat4 = [[f32; 4]; 3];

type MAT4 = mat4;

const unimat: MAT4 = [[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.]];

fn NORMALIZE(v: &POS3D) -> POS3D {
    let mut len: f32;
    len = ((v).x*(v).x + (v).y*(v).y + (v).z*(v).z).sqrt();
	POS3D{x: (v).x / len, y: (v).y / len, z: (v).z / len, reserved: 0.}
}

fn concat(m: &MAT4, v: &POS3D) -> POS3D{
	let mut ret = POS3D{
        x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3],
        y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3],
        z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3],
        reserved: 0.
    };
	return ret;
}


fn matcat(m1: &MAT4, m2: &MAT4) -> MAT4{
	let mut ret: MAT4 = unimat;
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



pub fn render(ren: &mut RENDER, pointproc: &mut FnMut(i32, i32, &FCOLOR)) {
	let mut ix: i32;
    let mut iy: i32;
	let mut fc = FCOLOR{fred: 0., fgreen: 0., fblue: 0.};
	let mut view: MAT4 = unimat;

	ren.light = NORMALIZE(&ren.light);
	ren.vnm = NORMALIZE(&ren.vnm);

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
/*	view.x[0][3] = 100.;
	view.x[1][3] = 0.;
	view.x[2][3] = 30.;*/

	for iy in 0..ren.yres {
		for ix in 0..ren.xres {
			let vi = &ren.cam;
            let mut eye: POS3D = POS3D{ /* cast ray direction vector? */
			    x: 1.,
			    y: (ix - ren.xres / 2) as f32 * 2. * ren.xfov / ren.xres as f32,
			    z: (iy - ren.yres / 2) as f32 * 2. * ren.yfov / ren.yres as f32,
                reserved: 0.
            };
			eye = concat(&view, &eye);
			eye = NORMALIZE(&eye);

			//raytrace(ren, &vi, &eye, &fc, 0, 0);
            //println!("Writing {},{}", ix, iy);
			pointproc(ix, iy, &fc);
		}
	}
}


/* find first object the ray hits */
fn raycast(ren: &mut RENDER, vi: &POS3D, eye: &POS3D, pIdx: &mut i32,
    ig: &SOBJECT, flags: u32) -> f32
{
	let mut Idx: i32;
	let mut t0: f32;
    let mut w: f32;
    let mut t: f32;
    let mut wpt = POS3D{x: 0., y: 0., z: 0., reserved: 0.};

	t = std::f32::INFINITY;

	for (Idx, obj) in ren.objects.iter_mut().enumerate() {
        if ig as *const _ != obj as *const _ {
            let mut b: f32;
            let mut c: f32;
            let mut d: f32;

            /* calculate vector from eye position to the object's center. */
            wpt.x = vi.x - obj.org.x;
            wpt.y = vi.y - obj.org.y;
            wpt.z = vi.z - obj.org.z;

            /* scalar product of the ray and the vector. */
            b = 2.0f32 * (eye.x * wpt.x + eye.y * wpt.y + eye.z * wpt.z);

            /* ??? */
            c = wpt.x * wpt.x + wpt.y * wpt.y + wpt.z * wpt.z -
                    obj.r * obj.r;

            /* discriminant?? */
            d = b * b - 4.0f32 * c;
            if d >= std::f32::EPSILON {
                d = d.sqrt();
                t0 = (-b - d) as f32 / 2.0f32;
                if 0 == (flags & OUTONLY) && t0 >= 0.0f32 && t0 < t {
                    t = t0;
                    *pIdx = Idx as i32;
                }
                else if 0 == (flags & INONLY) && 0f32 < (t0 + d) && t0 + d < t {
                    t = t0 + d;
                    *pIdx = Idx as i32;
                }
            }
        }
    }

	wpt.x = vi.x - ren.objects[0].org.x;
	wpt.y = vi.y - ren.objects[0].org.y;
	wpt.z = vi.z - ren.objects[0].org.z;
	w = ren.vnm.x * eye.x + ren.vnm.y * eye.y + ren.vnm.z * eye.z;
	if /*fabs(w) > 1.0e-7*/ w <= 0. {
		t0 = (-ren.vnm.x * wpt.x - ren.vnm.y * wpt.y - ren.vnm.z * wpt.z) / w;
		if t0 >= 0. && t0 < t {
			t = t0;
			*pIdx = 0;
		}
	}

	return t;
}