use ffmpeg_next::format::Pixel;

use crate::{dtos::EmbedConfigs, errors::steg_service_error::StegServiceError};

// pub struct BufferGeneric {
//     pub reader: Option<BufReader<File>>,
//     pub writer: Option<BufWriter<File>>,
//     pub buffer: [u8; 1028],
//     pub bit_index: usize,
//     pub bits_read: usize,
//     pub payload_exhausted: bool,
// }

pub const BLOCKS_PER_MACROBLOCK: u8 = 4;

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

pub fn process_frame<F, T, W>(
    configs: &EmbedConfigs,
    state: &mut W,
    buffer: &mut T,
    frame: &mut ffmpeg_next::frame::Video,
    channel_method: F,
) -> Result<(), StegServiceError>
where
    F: Fn(
        &EmbedConfigs,
        &mut W,
        &mut T,
        &mut ffmpeg_next::frame::Video,
        u32,
        u32,
        usize,
    ) -> Result<(), StegServiceError>,
{
    //allows to add more codecs, eg RGB also allows prioritizing best channel to embed in by
    //changing order of if statements for sensible defaults
    if let Some(yuv) = &configs.channels_to_embed.yuv {
        let (y_wdith, y_height, u_width, u_height, v_width, v_height) =
            find_dimensions_for_codec(frame, configs)?;

        if yuv.y {
            channel_method(configs, state, buffer, frame, y_wdith, y_height, Y_PLANE)?;
        }
        if yuv.cb {
            channel_method(configs, state, buffer, frame, u_width, u_height, CB_PLANE)?;
        }
        if yuv.cr {
            channel_method(configs, state, buffer, frame, v_width, v_height, CR_PLANE)?;
        }
    } else {
        return Err(StegServiceError::UnsupportedCodec);
    }
    Ok(())
}
