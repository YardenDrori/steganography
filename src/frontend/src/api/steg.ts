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

export async function embed(
  access_token: string,
  request: EmbedFileRequest,
): Promise<number> {
  return await axios.post(`${BASE_URL}/api/embed/video`, request, {
    headers: { Authorization: `Bearer ${access_token}` },
  });
}
