use std::{error::Error, io, ops::Deref};

use crate::{config, strokes, utils};
use gtk4::{gio, gsk};
use image::{io::Reader, GenericImageView};
use serde::{Deserialize, Serialize};

use super::StrokeBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitmapImage {
    pub data_base64: String,
    pub bounds: p2d::bounding_volume::AABB,
    pub intrinsic_size: na::Vector2<f64>,
    #[serde(skip, default = "utils::default_caironode")]
    pub caironode: gsk::CairoNode,
}

impl StrokeBehaviour for BitmapImage {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.bounds = self
            .bounds
            .transform_by(&na::geometry::Isometry2::new(offset, 0.0));
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        self.bounds = new_bounds;
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let mut cx = tera::Context::new();

        /*         let x = self.bounds.mins[0] + offset[0];
        let y = self.bounds.mins[1] + offset[1];
        let width = self.bounds.maxs[0] - self.bounds.mins[0];
        let height = self.bounds.maxs[1] - self.bounds.mins[1]; */
        let x = 0.0;
        let y = 0.0;
        let width = self.intrinsic_size[0];
        let height = self.intrinsic_size[1];

        cx.insert("x", &x);
        cx.insert("y", &y);
        cx.insert("width", &width);
        cx.insert("height", &height);
        cx.insert("data_base64", &self.data_base64);

        let templ = String::from_utf8(
            gio::resources_lookup_data(
                (String::from(config::APP_IDPATH) + "templates/bitmapimage.svg.templ").as_str(),
                gio::ResourceLookupFlags::NONE,
            )?
            .deref()
            .to_vec(),
        )
        .unwrap();

        let svg = tera::Tera::one_off(templ.as_str(), &cx, false)?;

        let intrinsic_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![self.intrinsic_size[0], self.intrinsic_size[1]],
        );

        let bounds = p2d::bounding_volume::AABB::new(
            na::point![
                self.bounds.mins[0] + offset[0],
                self.bounds.mins[1] + offset[1]
            ],
            na::point![
                self.bounds.maxs[0] + offset[0],
                self.bounds.maxs[1] + offset[1]
            ],
        );

        let svg = strokes::wrap_svg(
            svg.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );

        Ok(svg)
    }

    fn update_caironode(&mut self, scalefactor: f64) {
        if let Ok(caironode) = self.gen_caironode(scalefactor) {
            self.caironode = caironode;
        } else {
            log::error!("failed to gen_caironode() in update_caironode() of markerstroke");
        }
    }

    fn gen_caironode(&self, scalefactor: f64) -> Result<gsk::CairoNode, Box<dyn Error>> {
        strokes::gen_caironode_for_svg(
            self.bounds,
            scalefactor,
            strokes::add_xml_header(self.gen_svg_data(na::vector![0.0, 0.0])?.as_str()).as_str(),
        )
    }
}

impl BitmapImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 28.0;
    pub const OFFSET_Y_DEFAULT: f64 = 28.0;

    pub fn import_from_image_bytes<P>(
        to_be_read: P,
        pos: na::Vector2<f64>,
    ) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<[u8]>,
    {
        let reader = Reader::new(io::Cursor::new(&to_be_read)).with_guessed_format()?;

        let bitmap_data = reader.decode()?;
        let dimensions = bitmap_data.dimensions();
        let intrinsic_size = na::vector![f64::from(dimensions.0), f64::from(dimensions.1)];

        let bounds = p2d::bounding_volume::AABB::new(
            na::Point2::from(pos),
            na::Point2::from(intrinsic_size + pos),
        );
        let data_base64 = base64::encode(&to_be_read);

        Ok(Self {
            data_base64,
            bounds,
            intrinsic_size,
            caironode: utils::default_caironode(),
        })
    }

    /*     pub fn import_from_pdf_bytes<P>(
        to_be_read: P,
        pos: na::Vector2<f64>,
    ) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<[u8]>,
    {

        Ok(())
    } */
}
