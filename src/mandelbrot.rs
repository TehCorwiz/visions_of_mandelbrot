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
    iteration_counts: Vec<Vec<f64>>,
    frame_buffer: Vec<u8>,
    palette: Vec<LinSrgb>,
    recalculate: bool,
    redraw: bool,
    drawing: bool,
}

impl MandelbrotSet {
    pub(crate) fn new(width: usize, height: usize) -> MandelbrotSet {
        let max_iterations = 1000.0;

        Self {
            width,
            height,
            max_iterations,
            x_scale_min: -2.00,
            x_scale_max: 0.47,
            y_scale_min: -1.12,
            y_scale_max: 1.12,
            iteration_counts: vec![vec![0.0; width]; height],
            frame_buffer: vec![0xff as u8; width * height * 4],
            palette: MandelbrotSet::rainbow_palette(max_iterations as usize),
            recalculate: true,
            redraw: true,
            drawing: false,
        }
    }

    pub(crate) fn update(&mut self) {
        // TODO: Interactivity
    }

    pub(crate) fn randomize_palette(&mut self) {
        self.palette = MandelbrotSet::random_palette(self.max_iterations as usize);
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
        self.recalculate = true;
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

        self.recalculate = true;
        self.redraw = true;
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub(crate) fn draw(&mut self, frame: &mut [u8]) {
        self.drawing = true;

        if self.recalculate {
            self.recalculate = false;

            // Counts the number of iterations in each pixel location.
            self.iteration_counts = vec![vec![0.0; self.width]; self.height];

            for (y, row) in self.iteration_counts.iter_mut().enumerate() {
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
        }

        if self.redraw {
            self.redraw = false;
            self.draw_to_frame_buffer();
        }

        frame.copy_from_slice(&self.frame_buffer);

        self.drawing = false;
    }

    // The problem I'm going to have is that at higher zoom levels the x and y scale values will get
    //  too small. I'll effectively hit the float resolution limit. Maybe use a library to provide
    //  arbitrary resolution numbers?
    fn draw_to_frame_buffer(&mut self) {
        for (i, pixel) in self.frame_buffer.chunks_exact_mut(4).enumerate() {
            let x = i % self.width as usize;
            let y = i / self.width as usize;

            let rgba: [u8; 4] = if self.iteration_counts[y][x] == self.max_iterations {
                [0, 0, 0, 0xff]
            } else {
                let iterations: usize = self.iteration_counts[y][x].floor() as usize;
                let fraction = self.iteration_counts[y][x] % 1.0;

                let color1 = self.palette[iterations];
                let color2 = self.palette[iterations + 1];

                MandelbrotSet::color_to_rgba(&Gradient::from([
                    (0.0, color1),
                    (1.0, color2)
                ]).get(fraction as f32))
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn random_palette(n_colors: usize) -> Vec<LinSrgb> {
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

    fn rainbow_palette(n_colors: usize) -> Vec<LinSrgb> {
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
            let log_2 = 2.0_f64.log10();
            let nu = (log_zn / log_2).log10() / log_2;
            iteration = iteration + 1.0 - nu;
        }

        iteration
    }
}
