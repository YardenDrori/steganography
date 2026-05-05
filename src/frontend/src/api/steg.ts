import axios from "axios";

const BASE_URL = "http://localhost:3000";

export type YUVChannels = { y: boolean; cb: boolean; cr: boolean };
export type RGBChannels = { r: boolean; g: boolean; b: boolean };
export type Channels = { yuv?: YUVChannels; rgb?: RGBChannels };
export type EmbedMethod = "QIM" | "STDM" | "SS" | "ISS";

export type EmbedConfigs = {
  channels_to_embed: Channels;
  coefficients_to_embed: boolean[];
  coefficients_per_bit: number;
  blocks_per_macroblock: number;
  delta: number;
  seed?: string;
  method: EmbedMethod;
  reed_solomon_padding_byte_count: number;
};

export type EmbedFileRequest = {
  payload_id: number;
  carrier_id: number;
  configs: EmbedConfigs;
  password?: string;
};

export type ExtractFileRequest = {
  steg_object_id: number;
  configs: EmbedConfigs;
  password?: string;
};

export type StegFileResponse = {
  id: number;
  filename: string;
  created_at: string;
  is_carrier: boolean;
  is_steg_object: boolean;
};

export async function embed(
  access_token: string,
  request: EmbedFileRequest,
): Promise<StegFileResponse> {
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

export async function extract(
  access_token: string,
  request: ExtractFileRequest,
): Promise<StegFileResponse> {
  try {
    const response = await axios.post(`${BASE_URL}/api/extract/video`, request, {
      headers: { Authorization: `Bearer ${access_token}` },
    });
    return response.data;
  } catch (err) {
    console.log(err);
    throw err;
  }
}
