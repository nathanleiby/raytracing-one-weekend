use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use rand::{random, Rng};

// Constants
const INF: f64 = f64::INFINITY;
const PI: f64 = 3.14159265358979323846264338327950288f64;

pub const COLOR_BLACK: Color = Color {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

const COLOR_WHITE: Color = Color {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

// Utility
fn degrees_to_radians(degrees: f64) -> f64 {
    degrees / 180.0 * PI
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn new_random() -> Vec3 {
        let mut rng = rand::thread_rng();

        Vec3 {
            x: rng.gen::<f64>(),
            y: rng.gen::<f64>(),
            z: rng.gen::<f64>(),
        }
    }

    pub fn new_random_bounded(min: f64, max: f64) -> Vec3 {
        let mut rng = rand::thread_rng();

        Vec3 {
            x: rng.gen_range(min..max),
            y: rng.gen_range(min..max),
            z: rng.gen_range(min..max),
        }
    }

    pub fn new_random_in_unit_sphere() -> Vec3 {
        loop {
            let v = Self::new_random();
            if dot(v, v) < 1.0 {
                return v;
            }
        }
    }

    pub fn new_random_in_hemisphere(normal: Vec3) -> Vec3 {
        let v = Self::new_random_in_unit_sphere();
        if dot(v, normal) > 0.0 {
            return v;
        }
        -v
    }

    pub fn new_random_unit_vector() -> Vec3 {
        Self::new_random_in_unit_sphere().unit_vector()
    }

    pub fn length(self) -> f64 {
        f64::sqrt(self.length_squared())
    }

    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn unit_vector(self) -> Vec3 {
        self / self.length()
    }

    pub fn near_zero(self) -> bool {
        let s = 1e-8;
        f64::abs(self.x) < s && f64::abs(self.y) < s && f64::abs(self.z) < s
    }
}

pub fn cross(u: Vec3, v: Vec3) -> Vec3 {
    Vec3 {
        x: u.y * v.z - u.z * v.y,
        y: u.z * v.x - u.x * v.z,
        z: u.x * v.y - u.y * v.x,
    }
}

pub fn dot(u: Vec3, v: Vec3) -> f64 {
    u.x * v.x + u.y * v.y + u.z * v.z
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - (n * 2.0 * dot(v, n))
}

pub fn refract(R: Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
    // fmin(dot(-unit_direction, rec.normal), 1.0);
    let cos_theta = f64::min(dot(-R, n), 1.0);
    let r_out_perp = etai_over_etat * (R + cos_theta * n);

    let r_out_parallel = -f64::sqrt(f64::abs(1.0 - r_out_perp.length_squared())) * n;

    return r_out_perp + r_out_parallel;
}

// Operator Overloading via Traits
impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Multiplication with scalar
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, f: f64) -> Vec3 {
        Vec3 {
            x: self.x * f,
            y: self.y * f,
            z: self.z * f,
        }
    }
}

// Multiplication with scalar (other direction)
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        v * self // re-use above def
    }
}

// Multiplication with vector
impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, f: f64) -> Vec3 {
        self * (1.0 / f)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Vec3 {
        Vec3 {
            x: self.x * -1.0,
            y: self.y * -1.0,
            z: self.z * -1.0,
        }
    }
}

pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Debug)]
pub struct Ray {
    orig: Point3,
    dir: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Ray {
        Ray { orig, dir }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }

    pub fn color(self, world: &mut impl Hittable, depth: i32) -> Color {
        // If we've exceeded the ray bounce limit, no more light is gathered.
        if depth <= 0 {
            return COLOR_BLACK;
        }

        let unit_direction = self.dir.unit_vector();
        match world.hit(&self, 0.001, INF) {
            Some(rec) => {
                let out = rec.mat_ptr.scatter(&self, &rec);
                match out {
                    Some(out) => out.attenuation * out.scattered.color(world, depth - 1),
                    None => COLOR_BLACK,
                }
            }
            None => {
                let t = 0.5 * (unit_direction.y + 1.0);
                COLOR_WHITE * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
            }
        }
    }
}

pub struct HitRecord {
    p: Point3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    mat_ptr: Rc<dyn Material>,
}

impl HitRecord {
    fn with_face_normal(self: Self, r: &Ray, outward_normal: Vec3) -> HitRecord {
        let front_face = dot(r.dir, outward_normal) < 0.0;
        let normal = if self.front_face {
            outward_normal
        } else {
            outward_normal
        };

        HitRecord {
            p: self.p,
            normal,
            t: self.t,
            front_face,
            mat_ptr: Rc::clone(&self.mat_ptr),
        }
    }
}

pub struct ScatterResult {
    pub scattered: Ray,
    pub attenuation: Color,
}

pub trait Material {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<ScatterResult>;
}

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let random_scatter_direction = rec.normal + Vec3::new_random_unit_vector();
        let scatter_direction = if random_scatter_direction.near_zero() {
            rec.normal
        } else {
            random_scatter_direction
        };

        Some(ScatterResult {
            scattered: Ray::new(rec.p, scatter_direction),
            attenuation: self.albedo,
        })
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let reflected = reflect(r.dir.unit_vector(), rec.normal);

        let scattered = Ray::new(
            rec.p,
            reflected + Vec3::new_random_in_unit_sphere() * self.fuzz,
        );
        if dot(scattered.dir, rec.normal) > 0.0 {
            Some(ScatterResult {
                scattered,
                attenuation: self.albedo,
            })
        } else {
            None
        }
    }
}

pub struct Dialectric {
    index_of_refraction: f64,
}

impl Dialectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }
}

impl Material for Dialectric {
    fn scatter(&self, r: &Ray, rec: &HitRecord) -> Option<ScatterResult> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };

        let unit_direction = r.dir.unit_vector();

        let cos_theta = f64::min(dot(-unit_direction, rec.normal), 1.0);
        let sin_theta = f64::sqrt(1.0 - cos_theta * cos_theta);

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract {
            reflect(unit_direction, rec.normal)
        } else {
            refract(unit_direction, rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);
        Some(ScatterResult {
            scattered,
            attenuation: COLOR_WHITE,
        })
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat_ptr: Rc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.orig - self.center;
        let a = dot(ray.dir, ray.dir);
        let half_b = dot(oc, ray.dir);
        let c = dot(oc, oc) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = f64::sqrt(discriminant);

        // try first root.. does it fall in time range?
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            // try 2nd root
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let t = root;
        let p = ray.at(t);
        let hr = HitRecord {
            t,
            p,
            normal: (p - self.center) / self.radius,
            front_face: false,
            mat_ptr: Rc::clone(&self.mat_ptr),
        };
        let outward_normal = (p - self.center) / self.radius;

        Some(HitRecord::with_face_normal(hr, ray, outward_normal))
    }
}

pub struct HitList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HitList {
    pub fn new() -> HitList {
        HitList {
            objects: Vec::new(),
        }
    }
    pub fn clear(mut self: Self) {
        self.objects.clear();
    }
    pub fn add(self: &mut Self, obj: Box<dyn Hittable>) {
        self.objects.push(obj);
    }
}

impl Hittable for HitList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // let mut closest_so_far = None;
        let hits = self.objects.iter().map(|obj| obj.hit(ray, t_min, t_max));
        let closest = hits.into_iter().min_by(|x, y| match (x, y) {
            (None, None) => Ordering::Greater,
            (Some(_x), None) => Ordering::Less,
            (None, Some(_y)) => Ordering::Greater,
            (Some(x), Some(y)) => x.t.total_cmp(&y.t),
        });

        closest.unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64) -> Camera {
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let origin = Point3::new(0.0, 0.0, 0.0);
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - (horizontal / 2.0) - (vertical / 2.0) - Vec3::new(0.0, 0.0, focal_length);

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        )
    }
}

pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x > max {
        return max;
    } else if x < min {
        return min;
    }
    x
}
