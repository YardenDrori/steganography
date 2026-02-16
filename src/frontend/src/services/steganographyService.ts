import { apiUpload } from './api';

export interface EmbedImageResponse {
  message: string;
  output_path?: string;
}

export const steganographyService = {
  async embedImage(imageFile: File, message: string) {
    const formData = new FormData();
    formData.append('image', imageFile);
    formData.append('message', message);

    return apiUpload<EmbedImageResponse>('/api/embed/image', formData);
  },
};
