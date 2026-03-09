use ffmpeg_next::Codec;
use ffmpeg_next::format::Pixel;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

// pub struct BufferGeneric {
//     pub reader: Option<BufReader<File>>,
//     pub writer: Option<BufWriter<File>>,
//     pub buffer: [u8; 1028],
//     pub bit_index: usize,
//     pub bits_read: usize,
//     pub payload_exhausted: bool,
// }

const Y_PLANE: usize = 0;
const CB_PLANE: usize = 1;
const CR_PLANE: usize = 2;

pub fn find_dimensions_for_codec(
    frame: &mut ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
) -> Result<(u32, u32, u32, u32, u32, u32), StegServiceError> {
    if configs.channels_to_embed.yuv.is_some() {
        // (y_width_div, y_height_div, cb_width_div, cb_height_div, cr_width_div, cr_height_div)
        let (y_width_div, y_height_div, cb_width_div, cb_height_div, cr_width_div, cr_height_div): (
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
    ) = match frame.format() {
        Pixel::YUV420P => return Ok((frame.width()/1, frame.height()/1, frame.width()/2, frame.height()/2, frame.width()/2, frame.height()/2)),
        Pixel::YUV422P => return Ok((frame.width()/1, frame.height()/1, frame.width()/2, frame.height()/1, frame.width()/2, frame.height()/1)),
        Pixel::YUV444P => return Ok((frame.width()/1, frame.height()/1, frame.width()/1, frame.height()/1, frame.width()/1, frame.height()/1)),
        _ => return Err(StegServiceError::UnsupportedCodec),
    };
    }
    Err(StegServiceError::UnsupportedCodec)
}

pub fn process_frame<F, T>(
    frame: &mut ffmpeg_next::frame::Video,
    configs: &EmbedConfigs,
    buffer: &mut T,
    channel_method: F,
) -> Result<(), StegServiceError>
where
    F: Fn(
        &mut ffmpeg_next::frame::Video,
        &EmbedConfigs,
        &mut T,
        usize,
        u32,
        u32,
    ) -> Result<(), StegServiceError>,
{
    //allows to add more codecs, eg RGB also allows prioritizing best channel to embed in by
    //changing order of if statements for sensible defaults
    if let Some(yuv) = &configs.channels_to_embed.yuv {
        let (y_wdith, y_height, u_width, u_height, v_width, v_height) =
            find_dimensions_for_codec(frame, configs)?;

        if yuv.y {
            channel_method(frame, configs, buffer, Y_PLANE, y_wdith, y_height)?;
        }
        if yuv.cb {
            channel_method(frame, configs, buffer, CB_PLANE, u_width, u_height)?;
        }
        if yuv.cr {
            channel_method(frame, configs, buffer, CR_PLANE, v_width, v_height)?;
        }
    } else {
        return Err(StegServiceError::UnsupportedCodec);
    }
    Ok(())
}
