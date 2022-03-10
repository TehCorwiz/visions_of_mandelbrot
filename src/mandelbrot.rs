use palette::{Gradient, LinSrgb};
use rand::Rng;

fn normalize(n: f64, r_min: f64, r_max: f64, t_min: f64, t_max: f64) -> f64 {
    (((n - r_min) / (r_max - r_min)) * (t_max - t_min)) + t_min
}

pub(crate) struct MandelbrotGenerator {
    width: usize,
    height: usize,
    max_iterations: f64,
    x_scale_min: f64,
    x_scale_max: f64,
    y_scale_min: f64,
    y_scale_max: f64,
    iteration_counts: Vec<Vec<f64>>,
    current_x: usize,
    current_y: usize,
    recalculate: bool,
}

impl MandelbrotGenerator {
    pub const DEFAULT_MAX_ITERATIONS: f64 = 1000.0;

    pub(crate) fn new(width: usize, height: usize, max_iterations: f64) -> MandelbrotGenerator {
        MandelbrotGenerator {
            width,
            height,
            max_iterations,
            x_scale_min: -2.00,
            x_scale_max: 0.47,
            y_scale_min: -1.12,
            y_scale_max: 1.12,
            iteration_counts: vec![vec![0.0; width]; height],
            current_x: 0,
            current_y: 0,
            recalculate: true,
        }
    }


    fn x_range(&self) -> f64 {
        (self.x_scale_max - self.x_scale_min).abs()
    }

    fn y_range(&self) -> f64 {
        (self.y_scale_max - self.y_scale_min).abs()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.resize_scaling_factors(width, height);
        self.width = width;
        self.height = height;
        self.iteration_counts = vec![vec![0.0; width]; height];
        self.recalculate();
    }

    fn resize_scaling_factors(&mut self, width: usize, height: usize) {
        let x_ratio = width as f64 / self.width as f64;
        let y_ratio = height as f64 / self.height as f64;

        let x_range = self.x_range();
        let y_range = self.y_range();

        let new_x_range_diff = (x_ratio * x_range) - x_range;
        let new_y_range_diff = (y_ratio * y_range) - y_range;

        self.x_scale_min -= new_x_range_diff / 2.0;
        self.x_scale_max += new_x_range_diff / 2.0;

        self.y_scale_min -= new_y_range_diff / 2.0;
        self.y_scale_max += new_y_range_diff / 2.0;
    }

    pub(crate) fn zoom(&mut self, coords: (f32, f32), factor: f64) {
        let x_range = self.x_range();
        let y_range = self.y_range();

        let new_x_range = x_range * factor;
        let new_y_range = y_range * factor;

        let new_midpoint_x = normalize(
            coords.0 as f64,
            0.0,
            self.width as f64,
            self.x_scale_min,
            self.x_scale_max,
        );
        let new_midpoint_y = normalize(
            coords.1 as f64,
            0.0,
            self.height as f64,
            self.y_scale_min,
            self.y_scale_max,
        );

        self.x_scale_min = new_midpoint_x - (new_x_range / 2.0);
        self.x_scale_max = new_midpoint_x + (new_x_range / 2.0);

        self.y_scale_min = new_midpoint_y - (new_y_range / 2.0);
        self.y_scale_max = new_midpoint_y + (new_y_range / 2.0);

        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        self.recalculate = true;
    }

    fn test_pixel(&self, px: u32, py: u32) -> f64 {
        let x0 = normalize(
            px as f64,
            0.0,
            (self.width - 1) as f64,
            self.x_scale_min,
            self.x_scale_max,
        );

        let y0 = normalize(
            py as f64,
            0.0,
            (self.height - 1) as f64,
            self.y_scale_min,
            self.y_scale_max,
        );

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut x2: f64 = 0.0;
        let mut y2: f64 = 0.0;

        let mut iteration = 0.0;

        // Cardioid checking
        let y0_2 = y0 * y0;
        let p = ((x0 - 0.25).powf(2.0) + y0_2).sqrt();

        let is_large_cardioid = x0 <= p - 2.0 * p * p + 0.25;
        let is_period_2_bulb = (x0 + 1.0).powf(2.0) + y0_2 <= 1.0 / 16.0;

        if is_large_cardioid || is_period_2_bulb {
            return self.max_iterations;
        }

        let mut x_old = 0.0;
        let mut y_old = 0.0;
        let mut period = 0;

        // Escape algorithm
        while ((x2 + y2) <= 4.0) && iteration < self.max_iterations {
            y = 2.0 * x * y + y0;
            x = x2 - y2 + x0;
            x2 = x * x;
            y2 = y * y;

            iteration += 1.0;

            // Periodicity checking
            if x == x_old && y == y_old {
                return self.max_iterations;
            }

            period += 1;
            if period > 20 {
                period = 0;
                x_old = x;
                y_old = y;
            }
        }

        if iteration < self.max_iterations {
            let log_zn = (x2 + y2).log10();
            let log_2 = 2.0_f64.log10();
            let nu = (log_zn / log_2).log10() / log_2;
            iteration = iteration + 1.0 - nu;
        }

        iteration
    }
}

impl Iterator for MandelbrotGenerator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.current_x;
        let y = self.current_y;

        if self.recalculate {
            self.iteration_counts[y][x] = self.test_pixel(x as u32, y as u32);
        }

        self.current_x += 1;

        if self.current_x >= self.width {
            self.current_x = 0;
            self.current_y += 1;
        }

        if self.current_y >= self.height {
            self.current_y = 0;
            self.current_x = 0;
            self.recalculate = false;
        }

        Some(self.iteration_counts[y][x])
    }
}

pub(crate) struct MandelbrotRenderer {
    pub(crate) generator: MandelbrotGenerator,
    width: usize,
    height: usize,
    pub(crate) palette: Vec<LinSrgb>,
    redraw: bool,
    frame_buffer: Vec<u8>,
}

impl MandelbrotRenderer {
    pub(crate) fn new(width: usize, height: usize, generator: MandelbrotGenerator) -> Self {
        MandelbrotRenderer {
            generator,
            width,
            height,
            palette: MandelbrotRenderer::rainbow_palette(MandelbrotGenerator::DEFAULT_MAX_ITERATIONS as usize),
            redraw: true,
            frame_buffer: vec![0xffu8; width * height * 4],
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut [u8]) {
        if self.redraw {
            self.redraw = false;
            self.draw_to_frame_buffer();
        }

        frame.copy_from_slice(&self.frame_buffer);

        self.redraw = false;
    }

    fn draw_to_frame_buffer(&mut self) {
        for pixel in self.frame_buffer.chunks_exact_mut(4) {
            let mandelbrot_value = self.generator.next().unwrap();
            let rgba: [u8; 4] = if mandelbrot_value == self.generator.max_iterations {
                [0, 0, 0, 0xff]
            } else {
                let iterations: usize = mandelbrot_value.floor() as usize;
                let fraction = mandelbrot_value % 1.0;

                let color1 = self.palette[iterations];
                let color2 = self.palette[iterations + 1];

                MandelbrotRenderer::color_to_rgba(&Gradient::from([
                    (0.0, color1),
                    (1.0, color2)
                ]).get(fraction as f32))
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    pub(crate) fn zoom(&mut self, coords: (f32, f32), factor: f64) {
        self.generator.zoom(coords, factor);
        self.redraw = true;
    }

    pub(crate) fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.frame_buffer = vec![0xffu8; width * height * 4];
        self.generator.resize(width, height);
        self.redraw = true;
    }

    pub(crate) fn randomize_palette(&mut self) {
        self.palette = MandelbrotRenderer::random_palette(self.generator.max_iterations as usize);
        self.redraw = true;
    }

    pub(crate) fn random_palette(n_colors: usize) -> Vec<LinSrgb> {
        let mut rng = rand::thread_rng();
        let mut pool: Vec<f32> = vec![0.0; 15];
        for i in 1..15 {
            assert!(i < pool.len());
            pool[i] = rng.gen_range(0.0..1.0)
        }

        Gradient::from(vec![
            (0.0, LinSrgb::new(pool.pop().unwrap(), pool.pop().unwrap(), pool.pop().unwrap())),
            (0.1, LinSrgb::new(pool.pop().unwrap(), pool.pop().unwrap(), pool.pop().unwrap())),
            (2.5, LinSrgb::new(pool.pop().unwrap(), pool.pop().unwrap(), pool.pop().unwrap())),
            (6.0, LinSrgb::new(pool.pop().unwrap(), pool.pop().unwrap(), pool.pop().unwrap())),
            (10.0, LinSrgb::new(pool.pop().unwrap(), pool.pop().unwrap(), pool.pop().unwrap())),
        ]).take(n_colors).collect()
    }

    pub(crate) fn rainbow_palette(n_colors: usize) -> Vec<LinSrgb> {
        Gradient::from(vec![
            (0.0, LinSrgb::new(1.0, 0.0, 0.0)),
            (0.05, LinSrgb::new(0.0, 1.0, 0.0)),
            (0.5, LinSrgb::new(0.0, 0.0, 1.0)),
            (1.5, LinSrgb::new(0.0, 1.0, 0.0)),
            (2.5, LinSrgb::new(1.0, 0.0, 0.0)),
        ]).take(n_colors).collect()
    }

    fn color_to_rgba(color: &LinSrgb) -> [u8; 4] {
        [
            (color.red * 0xff as f32) as u8,
            (color.green * 0xff as f32) as u8,
            (color.blue * 0xff as f32) as u8,
            0xff,
        ]
    }
}
