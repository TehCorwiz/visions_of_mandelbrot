use palette::{Gradient, LinSrgb};
use rand::Rng;

fn normalize(n: &f64, r_min: &f64, r_max: &f64, t_min: &f64, t_max: &f64) -> f64 {
    (((n - r_min) / (r_max - r_min)) * (t_max - t_min)) + t_min
}

pub(crate) struct MandelbrotSet {
    width: usize,
    height: usize,
    max_iterations: f64,
    x_scale_min: f64,
    x_scale_max: f64,
    y_scale_min: f64,
    y_scale_max: f64,
    frame_buffer: Vec<u8>,
    palette: Gradient<LinSrgb>,
    redraw: bool,
    drawing: bool,
}

impl MandelbrotSet {
    pub(crate) fn new(width: usize, height: usize) -> MandelbrotSet {
        Self {
            width,
            height,
            max_iterations: 1000.0,
            x_scale_min: -2.00,
            x_scale_max: 0.47,
            y_scale_min: -1.12,
            y_scale_max: 1.12,
            frame_buffer: vec![0xff as u8; width * height * 4],
            palette: MandelbrotSet::random_palette(),
            redraw: true,
            drawing: false,
        }
    }

    pub(crate) fn update(&mut self) {
        // TODO: Interactivity
    }

    pub(crate) fn randomize_palette(&mut self) {
        self.palette = MandelbrotSet::random_palette();
        self.redraw = true;
    }

    fn x_range(&self) -> f64 {
        (self.x_scale_max - self.x_scale_min).abs()
    }

    fn y_range(&self) -> f64 {
        (self.y_scale_max - self.y_scale_min).abs()
    }

    // Adjusts the x and y scale values such that the scale of the image remains constant
    //  regardless of resolution. This has the effect of expanding the canvas when increasing the
    //  window size and zooming in when narrowing the window size.
    // TODO: Consider adding the option to only adjust the aspect ratio.
    fn resize_scaling_factors(&mut self, width: usize, height: usize) {
        let x_ratio = width as f64 / self.width as f64;
        let y_ratio = height as f64 / self.height as f64;

        let x_range = self.x_range();
        let y_range = self.y_range();

        let new_x_range_diff = (x_ratio * x_range) - x_range;
        let new_y_range_diff = (y_ratio * y_range) - y_range;

        self.x_scale_min = self.x_scale_min - new_x_range_diff / 2.0;
        self.x_scale_max = self.x_scale_max + new_x_range_diff / 2.0;

        self.y_scale_min = self.y_scale_min - new_y_range_diff / 2.0;
        self.y_scale_max = self.y_scale_max + new_y_range_diff / 2.0;
    }

    pub(crate) fn resize(&mut self, width: usize, height: usize) {
        self.resize_scaling_factors(width, height);

        self.width = width;
        self.height = height;
        self.frame_buffer = vec![0xff as u8; width * height * 4];
        self.redraw = true;
    }

    pub(crate) fn zoom(&mut self, coords: (f32, f32), factor: f64) {
        let x_range = self.x_range();
        let y_range = self.y_range();

        let new_x_range = x_range * factor;
        let new_y_range = y_range * factor;

        let new_midpoint_x = normalize(
            &(coords.0 as f64),
            &0.0,
            &(self.width as f64),
            &self.x_scale_min,
            &self.x_scale_max,
        );
        let new_midpoint_y = normalize(
            &(coords.1 as f64),
            &0.0,
            &(self.height as f64),
            &self.y_scale_min,
            &self.y_scale_max,
        );

        self.x_scale_min = new_midpoint_x - (new_x_range / 2.0);
        self.x_scale_max = new_midpoint_x + (new_x_range / 2.0);

        self.y_scale_min = new_midpoint_y - (new_y_range / 2.0);
        self.y_scale_max = new_midpoint_y + (new_y_range / 2.0);

        self.redraw = true;
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub(crate) fn draw(&mut self, frame: &mut [u8]) {
        if self.redraw && !self.drawing {
            self.redraw = false;

            self.draw_to_frame_buffer();
        }

        frame.copy_from_slice(&self.frame_buffer);
    }

    // The problem I'm going to have is that at higher zoom levels the x and y scale values will get
    //  too small. I'll effectively hit the float resolution limit. Maybe use a library to provide
    //  arbitrary resolution numbers?
    fn draw_to_frame_buffer(&mut self) {
        self.drawing = true;

        // Counts the number of iterations in each pixel location.
        let mut iteration_counts: Vec<Vec<f64>> = vec![vec![0.0; self.width]; self.height];

        for (y, row) in iteration_counts.iter_mut().enumerate() {
            for (x, val) in row.iter_mut().enumerate() {
                let x = x as u32;
                let y = y as u32;

                *val = MandelbrotSet::test_pixel(
                    &x,
                    &y,
                    &self.width,
                    &self.height,
                    &self.max_iterations,
                    &self.x_scale_min,
                    &self.x_scale_max,
                    &self.y_scale_min,
                    &self.y_scale_max,
                );
            }
        }

        for (i, pixel) in self.frame_buffer.chunks_exact_mut(4).enumerate() {
            let x = i % self.width as usize;
            let y = i / self.width as usize;

            let rgba: [u8; 4] = if iteration_counts[y][x] == self.max_iterations as f64 {
                [0, 0, 0, 0xff]
            } else {
                let iterations = iteration_counts[y][x].floor();
                let fraction = iteration_counts[y][x] % 1.0;
                let gradient = Gradient::from([
                    (0.0, self.palette.get((iterations / self.max_iterations) as f32)),
                    (1.0, self.palette.get(((iterations + 1.0) / self.max_iterations) as f32)),
                ]);

                let color = gradient.get(fraction as f32);
                [
                    (color.red * 255.0) as u8,
                    (color.green * 255.0) as u8,
                    (color.blue * 255.0) as u8,
                    0xff,
                ]
            };

            pixel.copy_from_slice(&rgba);
        }

        self.drawing = false;
    }

    fn random_palette() -> Gradient<LinSrgb> {
        let mut rng = rand::thread_rng();

        Gradient::from(vec![
            (
                0.0,
                LinSrgb::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ),
            ),
            (
                0.5,
                LinSrgb::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ),
            ),
            (
                1.0,
                LinSrgb::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ),
            ),
        ])
    }

    // Returns the number of iterations to diverge.
    fn test_pixel(
        px: &u32,
        py: &u32,
        width: &usize,
        height: &usize,
        max_iterations: &f64,
        x_scale_min: &f64,
        x_scale_max: &f64,
        y_scale_min: &f64,
        y_scale_max: &f64,
    ) -> f64 {
        let x0 = normalize(
            &(*px as f64),
            &0.0,
            &((width - 1) as f64),
            x_scale_min,
            x_scale_max,
        );

        let y0 = normalize(
            &(*py as f64),
            &0.0,
            &((height - 1) as f64),
            y_scale_min,
            y_scale_max,
        );

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut x2: f64 = 0.0;
        let mut y2: f64 = 0.0;

        let mut iteration = 0.0;

        // Cardioid checking
        let p = ((x0 - 0.25).powf(2.0) + y0.powf(2.0)).sqrt();
        if x0 <= p - 2.0 * p.powf(2.0) + 0.25 {
            return *max_iterations; // Large cardioid
        } else if (x0 + 1.0).powf(2.0) + y0.powf(2.0) <= 1.0 / 16.0 {
            return *max_iterations; // Period-2 bulb
        }

        let mut x_old = 0.0;
        let mut y_old = 0.0;
        let mut period = 0;

        // Escape algorithm
        while ((x2 + y2) <= 4.0) && iteration < *max_iterations {
            y = 2.0 * x * y + y0;
            x = x2 - y2 + x0;
            x2 = x.powf(2.0);
            y2 = y.powf(2.0);

            iteration += 1.0;

            // Periodicity checking
            if x == x_old && y == y_old {
                return *max_iterations;
            }

            period += 1;
            if period > 20 {
                period = 0;
                x_old = x;
                y_old = y;
            }
        }

        if iteration < *max_iterations {
            let log_zn = (x2 + y2).log10();

            let nu = (log_zn / 2.0_f64.log10()).log10() / 2.0_f64.log10();
            iteration = iteration + 1.0 - nu;
        }

        iteration
    }
}
