use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use num_complex::ComplexDistribution;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand_distr::Uniform;
use std::thread;

type F = f64;
type C = Complex<F>;
type ImBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

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
    for _ in 0..=escape_threshold {
        if z.norm_sqr() > 4. {
            return false;
        }
        z = mandelbrot_iter(z, c);
    }
    return true;
}

fn channel(i: usize) -> usize {
    if i < 300 {
        0
    } else if i < 600 {
        1
    } else {
        2
    }
}

fn main() -> Result<(), image::ImageError> {
    let lowx: F = -2.;
    let highx: F = 1.5;
    let highy: F = 1.5;
    let lowy: F = -highy;
    let resolution: usize = 300;
    let spanx = highx - lowx;
    let spany = highy - lowy;
    let f_pixelsx = (spanx * (resolution as F)).floor();
    let f_pixelsy = (spany * (resolution as F)).floor();
    let u_pixelsx = f_pixelsx as usize;
    let u_pixelsy = f_pixelsy as usize;
    let n_traces: usize = 100_000_000;
    let n_threads = 4;
    let escape_threshold: usize = 2_000;
    let buddha_trace_length = 1_000;
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
    let distcplx = ComplexDistribution::new(distrx, distry);
    let mut handles = Vec::new();
    for _ in 0..n_threads {
        handles.push(thread::spawn(move || {
            let mut rng = SmallRng::from_entropy();
            let mut canvas: Vec<[u32; 3]> = vec![[0; 3]; u_pixelsx * u_pixelsy];
            for _ in 0..n_traces {
                // propose
                let proposal = distcplx.sample(&mut rng);

                if !is_in_mandel(proposal, escape_threshold) {
                    let mut z = C::new(0., 0.);
                    for i in 0..=buddha_trace_length {
                        z = mandelbrot_iter(z, proposal);
                        if z.norm_sqr() > 16. {
                            break;
                        }
                        let x = z.re;
                        let y = z.im;
                        if x < lowx || x >= highx {
                            continue;
                        }
                        let xbin = bin(x, lowx, spanx, f_pixelsx);
                        let xbin = xbin as usize;

                        if y < lowy || y >= highy {
                            continue;
                        }
                        let ybin = bin(y, lowy, spany, f_pixelsy);
                        let ybin = ybin as usize;
                        canvas[xbin + u_pixelsx * ybin][channel(i)] += 1;
                        let ybin = u_pixelsy - ybin - 1;
                        canvas[xbin + u_pixelsx * ybin][channel(i)] += 1;
                    }
                }
            }
            canvas
        }));
    }
    let sub_canvas : thread::Result<Vec<Vec<[u32;3]>>> = handles.into_iter().map(|x| x.join()).collect();
    let sub_canvas = sub_canvas.unwrap();
    let mut canvas = vec![[0; 3]; u_pixelsx * u_pixelsy];
    for s_canva in &sub_canvas {
        for (dest_arr,src_arr) in canvas.iter_mut().zip(s_canva) {
            for (dest_cell, src_cell) in dest_arr.iter_mut().zip(src_arr) {
                *dest_cell += src_cell;
            }
        }
    }
    let mut maxlum = [0;3];
    for arr in &canvas {
        for (maxlum_cell, arr_cell) in maxlum.iter_mut().zip(arr) {
            *maxlum_cell = std::cmp::max(*arr_cell, *maxlum_cell);
        }
    }
    let mut img = ImBuffer::new(u_pixelsy as _, u_pixelsx as _);
    for (i, l) in canvas.iter().enumerate() {
        let x = i % u_pixelsx;
        let y = i / u_pixelsx;
        let mut color = [0; 3];
        for channel in 0..3 {
            let ratio = (l[channel] as F) / (maxlum[channel] as F);
            color[channel] = (ratio.sqrt() * 255.) as u8;
        }
        // dbg!(lum);
        img.put_pixel(y as u32, x as u32, Rgb(color));
    }
    img.save("output.png")
}
