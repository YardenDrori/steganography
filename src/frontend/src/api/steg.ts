import axios from "axios";

const BASE_URL = "http://localhost:3000";

export type YUVChannels = {
  y: boolean;
  cb: boolean;
  cr: boolean;
};

export type RGBChannels = {
  r: boolean;
  g: boolean;
  b: boolean;
};

export type Channels = {
  yuv?: YUVChannels;
  rgb?: RGBChannels;
};

// coefficients_to_embed must be exactly 16 booleans (4x4 DCT grid)
export type EmbedConfigs = {
  channels_to_embed: Channels;
  coefficients_to_embed: boolean[];
  delta: number;
};

export type EmbedFileRequest = {
  payload_id: number;
  carrier_id: number;
  configs: EmbedConfigs;
};

export type EmbedResponse = {
  id: number;
  filename: string;
  created_at: string;
  is_carrier: boolean;
  is_steg_object: boolean;
};

export async function embed(
  access_token: string,
  request: EmbedFileRequest,
): Promise<EmbedResponse> {
  try {
    const response = await axios.post(`${BASE_URL}/api/embed/video`, request, {
      headers: { Authorization: `Bearer ${access_token}` },
    });
    return response.data;
  } catch (err) {
    console.log(err);
    throw err;
  }
}
