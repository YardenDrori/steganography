use ffmpeg_next::format::Pixel;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

pub struct BufferGeneric {
    pub reader: Option<BufReader<File>>,
    pub writer: Option<BufWriter<File>>,
    pub buffer: [u8; 1028],
    pub bit_index: usize,
    pub bits_read: usize,
}

pub fn process_frame<F>(
    frame: &mut ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    buffer: &mut BufferGeneric,
    channel_method: F,
) -> Result<(), StegServiceError>
where
    F: Fn(
        &mut ffmpeg_next::frame::Video,
        &EmbedConfigs,
        &mut BufferGeneric,
        usize,
        u32,
        u32,
    ) -> Result<(), StegServiceError>,
{
    const Y_PLANE: usize = 0;
    const CB_PLANE: usize = 1;
    const CR_PLANE: usize = 2;

    //allows to add more codecs, eng RGB also allows prioritizing best channel to embed in by
    //changing order of if statements for sensible defaults
    if let Some(yuv) = &configs.channels_to_embed.yuv {
        // (y_width_div, y_height_div, cb_width_div, cb_height_div, cr_width_div, cr_height_div)
        let (y_width_div, y_height_div, cb_width_div, cb_height_div, cr_width_div, cr_height_div): (u32, u32, u32, u32, u32, u32) = match frame.format() {
            Pixel::YUV420P => (1, 1, 2, 2, 2, 2),
            Pixel::YUV422P => (1, 1, 2, 1, 1, 1),
            Pixel::YUV444P => (1, 1, 1, 1, 1, 1),
            _ => return Err(StegServiceError::UnsupportedCodec),
        };
        if yuv.y {
            channel_method(
                frame,
                configs,
                buffer,
                Y_PLANE,
                frame.width() / y_width_div,
                frame.height() / y_height_div,
            )?;
        }
        if yuv.cb {
            channel_method(
                frame,
                configs,
                buffer,
                CB_PLANE,
                frame.width() / cb_width_div,
                frame.height() / cb_height_div,
            )?;
        }
        if yuv.cr {
            channel_method(
                frame,
                configs,
                buffer,
                CR_PLANE,
                frame.width() / cr_width_div,
                frame.height() / cr_height_div,
            )?;
        }
    } else {
        return Err(StegServiceError::UnsupportedCodec);
    }
    Ok(())
}
