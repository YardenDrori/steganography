import { apiRequest } from './api';

export interface FileMetadata {
  id: string;
  user_id: number;
  filename: string;
  content_type: string;
  size: number;
  created_at: string;
}

export interface PrepareUploadResponse {
  file_id: string;
  upload_url: string;
}

export const filesService = {
  async prepareUpload(filename: string, contentType: string, size: number) {
    return apiRequest<PrepareUploadResponse>('/api/files/prepare', {
      method: 'POST',
      body: JSON.stringify({ filename, content_type: contentType, size }),
    });
  },

  async confirmUpload(fileId: string) {
    return apiRequest<FileMetadata>(`/api/files/${fileId}/confirm`, {
      method: 'POST',
    });
  },

  async getFile(fileId: string) {
    return apiRequest<FileMetadata>(`/api/files/${fileId}`, {
      method: 'GET',
    });
  },

  async listFiles() {
    return apiRequest<FileMetadata[]>('/api/files', {
      method: 'GET',
    });
  },

  async deleteFile(fileId: string) {
    return apiRequest<{ message: string }>(`/api/files/${fileId}`, {
      method: 'DELETE',
    });
  },
};
