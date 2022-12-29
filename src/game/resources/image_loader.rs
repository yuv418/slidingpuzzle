use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use ggez::{graphics::Image, Context};
use lazy_static::lazy_static;

lazy_static! {
    static ref IMAGES: Mutex<Option<ImageLoader>> = Mutex::new(None);
}

#[derive(Default)]
pub struct ImageLoader {
    images: HashMap<String, Arc<Image>>,
    pub total: usize,
    pub loaded: usize,
}

impl ImageLoader {
    pub fn get_load_status(ctx: &mut Context) -> (usize, usize) {
        let mut images_b = IMAGES.lock().unwrap();
        if let None = *images_b {
            let mut loader = ImageLoader::default();
            loader.total = ctx.fs.read_dir("/images").expect("Failed to read dir").count();

            *images_b = Some(loader);
        }

        let images = images_b.as_mut().unwrap();
        if images.loaded < images.total {
            // Load next image.
            let img_texture = Image::from_path(ctx, format!("/images/{}.jpg", images.loaded)).expect("Failed to open image");
            images.images.insert(format!("{}", images.loaded), Arc::new(img_texture));
            images.loaded += 1;
        }
        (images.loaded, images.total)
    }

    pub fn get_img(num: usize) -> Option<Arc<Image>> {
        let images_b = IMAGES.lock().unwrap();
        let images = images_b.as_ref().unwrap();
        images.images.get(&format!("{}", num)).cloned()
    }

    // This exists for efficiency purposes
    // to avoid the Arc clone on get_img
    pub fn has_img(num: usize) -> bool {
        let images_b = IMAGES.lock().unwrap();
        let images = images_b.as_ref().unwrap();
        images.images.contains_key(&format!("{}", num))
    }
}
