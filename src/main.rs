use image::{ImageBuffer, Luma};
use num_complex::Complex;
use num_complex::ComplexDistribution;
use rand::prelude::*;
use rand_distr::Uniform;

type F = f32;
type C = Complex<F>;
type ImBuffer = ImageBuffer<Luma<u8>, Vec<u8>>;

fn bin(x: F, low: F, span: F, bins: F) -> F {
    let frac = (x - low) / span;
    (frac * bins).floor()
}

fn is_in_sub_mandelbrot(z: C) -> bool {
    // Cf https://fr.wikipedia.org/wiki/Ensemble_de_Mandelbrot#Cardio%C3%AFde_/_bourgeon_principal
    let x = z.re;
    let y = z.im;
    let maincard = {
        let psq = (x - 0.25).powi(2) + y.powi(2);
        let p = psq.sqrt();
        x < p - 2. * psq + 0.25
    };
    if maincard {
        return true;
    };
    let mainbdud = (x + 1.).powi(2) + y.powi(2) < 0.0625;
    mainbdud
}

fn mandelbrot_iter(z: C, c: C) -> C {
    z * z + c
}

fn is_in_mandel(c: C, escape_threshold: usize) -> bool {
    if is_in_sub_mandelbrot(c) {
        return true;
    }
    let mut z = C::new(0., 0.);
    for _ in 1..=escape_threshold {
        z = mandelbrot_iter(z, c);
        if z.norm_sqr() > 4. {
            return false;
        }
    }
    return true;
}

fn main() -> Result<(), image::ImageError> {
    let lowx: F = -2.;
    let highx: F = 1.;
    let lowy: F = -1.;
    let highy: F = 1.;
    let resolution: usize = 500;
    let spanx = highx - lowx;
    let spany = highy - lowy;
    let f_pixelsx = (spanx * (resolution as F)).floor();
    let f_pixelsy = (spany * (resolution as F)).floor();
    let u_pixelsx = f_pixelsx as usize;
    let u_pixelsy = f_pixelsy as usize;
    let n_traces: usize = 10_000_000;
    let escape_threshold: usize = 1_000;
    let buddha_trace_length = 10_000;
    println!(
        "Computing for {} <= x < {} and {} <= y < {}",
        lowx, highx, lowy, highy
    );
    println!(
        "Resolution {} => image pixel size: {} x {}",
        resolution, u_pixelsx, u_pixelsy
    );

    let distrx = Uniform::new(lowx, highx);
    let distry = Uniform::new(lowy, highy);
    let distcplx = ComplexDistribution::new(&distrx, &distry);
    let mut rng = thread_rng();
    let mut maxlum = 0;
    let mut canvas: Vec<u32> = vec![0; u_pixelsx * u_pixelsy];
    for _ in 0..n_traces {
        // propose
        let proposal = distcplx.sample(&mut rng);

        if !is_in_mandel(proposal, escape_threshold) {
            let mut z = C::new(0., 0.);
            for _ in 1..=buddha_trace_length {
                z = mandelbrot_iter(z, proposal);
                if z.norm_sqr() > 4. {
                    break;
                }
                let x = z.re;
                let y = z.im;
                let xbin = bin(x, lowx, spanx, f_pixelsx);
                if xbin < 0. || xbin >= f_pixelsx {
                    continue;
                }
                let xbin = xbin as usize;
                let ybin = bin(y, lowy, spany, f_pixelsy);
                if ybin < 0. || ybin >= f_pixelsy {
                    continue;
                }
                let ybin = ybin as usize;

                canvas[xbin + u_pixelsx * ybin] += 1;
                maxlum = std::cmp::max(canvas[xbin + u_pixelsx * ybin], maxlum);
            }
        }
    }
    let mut img = ImBuffer::new(u_pixelsy as _, u_pixelsx as _);
    for (i, l) in canvas.iter().enumerate() {
        let x = i % u_pixelsx;
        let y = i / u_pixelsx;
        let ratio = (*l as F) / (maxlum as F);
        let lum = ratio.sqrt() * 255.;
        // dbg!(lum);
        img.put_pixel(y as u32, x as u32, Luma([lum as u8]));
    }
    img.save("output.png")
}
