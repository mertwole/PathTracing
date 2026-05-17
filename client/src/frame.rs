use std::{hash::Hash, sync::Arc};

use futures::Stream;
use image::{Pixel, Rgb32FImage, RgbaImage};
use tokio::sync::{
    Mutex,
    watch::{Receiver, Sender, channel},
};
use tokio_stream::wrappers::WatchStream;

pub struct Frame {
    render_sum: Mutex<RenderSum>,
    result_sender: Sender<RgbaImage>,
    result_receiver: Receiver<RgbaImage>,
}

impl Hash for Frame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        //
    }
}

impl Frame {
    pub async fn new(width: u32, height: u32) -> Self {
        let (result_sender, result_receiver) = channel(RgbaImage::new(width, height));

        Self {
            render_sum: Mutex::from(RenderSum {
                sum: Rgb32FImage::new(width, height),
                count: 0,
            }),
            result_sender,
            result_receiver,
        }
    }

    pub async fn add_render(&self, render: Rgb32FImage) {
        // TODO: Check that width and height match.

        let mut render_sum = self.render_sum.lock().await;
        render_sum.add_render(render);

        let render_sum_clone = render_sum.clone();
        drop(render_sum);

        let image = render_sum_clone.into_image();
        let image = gamma_correction(image);
        self.result_sender.send(image).unwrap();
    }

    pub fn get_image_stream(self: Arc<Self>) -> impl Stream<Item = RgbaImage> {
        WatchStream::new(self.result_receiver.clone())
    }
}

#[derive(Clone)]
struct RenderSum {
    sum: Rgb32FImage,
    count: usize,
}

impl RenderSum {
    fn add_render(&mut self, render: Rgb32FImage) {
        for x in 0..render.width() {
            for y in 0..render.height() {
                let pixel = self.sum.get_pixel_mut(x, y);
                let rendered_pixel = render.get_pixel(x, y);
                pixel.0 = [0, 1, 2].map(|i| pixel.0[i] + rendered_pixel.0[i]);
            }
        }

        self.count += 1;
    }

    fn into_image(mut self) -> Rgb32FImage {
        let render_count = self.count.max(1);
        for x in 0..self.sum.width() {
            for y in 0..self.sum.height() {
                self.sum
                    .get_pixel_mut(x, y)
                    .apply(|ch| ch / render_count as f32);
            }
        }
        self.sum
    }
}

fn gamma_correction(image: Rgb32FImage) -> RgbaImage {
    let mut gamma_corrected_image = RgbaImage::new(image.width(), image.height());
    for x in 0..image.width() {
        for y in 0..image.height() {
            let res_pixel = image.get_pixel(x, y);
            let r = (res_pixel.0[0] / (1.0 + res_pixel.0[0]) * 255.0) as u8;
            let g = (res_pixel.0[1] / (1.0 + res_pixel.0[1]) * 255.0) as u8;
            let b = (res_pixel.0[2] / (1.0 + res_pixel.0[2]) * 255.0) as u8;
            let gc_pixel = gamma_corrected_image.get_pixel_mut(x, y);
            gc_pixel.0[0] = r;
            gc_pixel.0[1] = g;
            gc_pixel.0[2] = b;
            gc_pixel.0[3] = 255;
        }
    }

    gamma_corrected_image
}
