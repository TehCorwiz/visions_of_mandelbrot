use palette::{Gradient, LinSrgb};
use rand::Rng;

fn normalize(n: f64, r_min: f64, r_max: f64, t_min: f64, t_max: f64) -> f64 {
    (((n - r_min) / (r_max - r_min)) * (t_max - t_min)) + t_min
}

pub(crate) struct MandelbrotSet {
    width: usize,
    height: usize,
    max_iterations: u32,
    x_scale_min: f64,
    x_scale_max: f64,
    y_scale_min: f64,
    y_scale_max: f64,
    frame_buffer: Vec<u8>,
    palette: Vec<[u8; 4]>,
    redraw: bool,
    drawing: bool,
}

impl MandelbrotSet {
    pub(crate) fn new(width: usize, height: usize) -> MandelbrotSet {
        let frame_buffer = vec![0xff as u8; width * height * 4];
        let palette = MandelbrotSet::generate_palette();

        Self {
            width,
            height,
            max_iterations: 1000,
            x_scale_min: -2.00,
            x_scale_max: 0.47,
            y_scale_min: -1.12,
            y_scale_max: 1.12,
            frame_buffer,
            palette,
            redraw: true,
            drawing: false,
        }
    }

    pub(crate) fn update(&mut self) {
        // TODO: Interactivity
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
        let mut iteration_counts: Vec<Vec<u32>> = vec![vec![0; self.width]; self.height];
        // Counts the frequency of each iteration count.
        //  Iteration counts range from 1 to `self.max_iterations` and it's simpler to just have a
        //  +1 sized array than If we initialized it with just self.max_iterations and had to
        //  perform a -1 offset to query from the zero-indexed array (0..self.max_iterations - 1).
        let mut historgram: Vec<u32> = vec![0; (self.max_iterations + 1) as usize];

        for (y, row) in iteration_counts.iter_mut().enumerate() {
            for (x, val) in row.iter_mut().enumerate() {
                *val = MandelbrotSet::test_pixel(
                    x as u32,
                    y as u32,
                    self.width,
                    self.height,
                    self.max_iterations,
                    self.x_scale_min,
                    self.x_scale_max,
                    self.y_scale_min,
                    self.y_scale_max,
                );

                historgram[*val as usize] += 1;
            }
        }

        // Counts the total iterations
        let total: u32 = historgram.iter().sum();

        for (i, pixel) in self.frame_buffer.chunks_exact_mut(4).enumerate() {
            let x = i % self.width as usize;
            let y = i / self.width as usize;

            let rgba: [u8; 4] = if iteration_counts[y][x] == self.max_iterations {
                [0, 0, 0, 0xff]
            } else {
                let mut shade: f64 = 0.0;

                // histogram[0] is always = 0;
                assert!(historgram.len() >= iteration_counts[y][x] as usize);
                for n in 1..iteration_counts[y][x] {
                    shade += historgram[n as usize] as f64 / total as f64;
                }

                self.palette[(shade * self.palette.len() as f64) as usize]
            };

            pixel.copy_from_slice(&rgba);
        }

        self.drawing = false;
    }

    fn generate_palette() -> Vec<[u8; 4]> {
        let mut rng = rand::thread_rng();

        let gradient: Vec<LinSrgb> = Gradient::from([
            (0.0, LinSrgb::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))),
            (0.5, LinSrgb::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))),
            (1.0, LinSrgb::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))),
        ]).take(256).collect();

        let mut palette = vec![[0; 4]; 256];
        for (i, color) in gradient.iter().enumerate() {
            palette[i] = [
                (color.red * 0xff as f32) as u8,
                (color.green * 0xff as f32) as u8,
                (color.blue * 0xff as f32) as u8,
                0xff,
            ]
        }

        palette
    }

    // Returns the number of iterations to diverge.
    fn test_pixel(
        px: u32,
        py: u32,
        width: usize,
        height: usize,
        max_iterations: u32,
        x_scale_min: f64,
        x_scale_max: f64,
        y_scale_min: f64,
        y_scale_max: f64,
    ) -> u32 {
        let x0 = normalize(px as f64, 0.0, (width - 1) as f64, x_scale_min, x_scale_max);

        let y0 = normalize(
            py as f64,
            0.0,
            (height - 1) as f64,
            y_scale_min,
            y_scale_max,
        );

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut x2: f64 = 0.0;
        let mut y2: f64 = 0.0;

        let mut iteration: u32 = 0;

        while ((x.powf(2.0) + y.powf(2.0)) <= 4.0) && iteration < max_iterations {
            y = 2.0 * x * y + y0;
            x = x2 - y2 + x0;
            x2 = x.powf(2.0);
            y2 = y.powf(2.0);

            iteration += 1;
        }

        iteration
    }
}
