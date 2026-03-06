import axios from "axios";

const BASE_URL = "http://localhost:3000";

export type FileItem = { id: number; filename: string; created_at: string };
export type InitiateResponse = { upload_id: string; object_key: string };
export type PartInfo = { part_number: number; etag: string };
export type UploadPartResponse = { part: PartInfo };

export async function getFiles(accessToken: string) {
  try {
    return await axios.get(`${BASE_URL}/api/files/me`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function deleteFile(accessToken: string, fileId: number) {
  try {
    return await axios.delete(`${BASE_URL}/api/files/${fileId}`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function renameFile(
  accessToken: string,
  fileId: number,
  body: { new_name: string },
) {
  try {
    return await axios.patch(`${BASE_URL}/api/files/${fileId}`, body, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function initiateUpload(
  accessToken: string,
): Promise<InitiateResponse> {
  try {
    const response = await axios.post(`${BASE_URL}/api/files/initiate`, null, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return response.data;
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function uploadChunk(
  accessToken: string,
  objectKey: string,
  uploadId: string,
  partNumber: number,
  chunk: Blob,
): Promise<UploadPartResponse> {
  try {
    const response = await axios.post(
      `${BASE_URL}/api/files/upload-chunk?part_number=${partNumber}&upload_id=${uploadId}&object_key=${objectKey}`,
      chunk,
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return response.data;
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function completeUpload(
  accessToken: string,
  uploadId: string,
  objectKey: string,
  filename: string,
  parts: PartInfo[],
): Promise<FileItem> {
  try {
    const response = await axios.post(
      `${BASE_URL}/api/files/complete`,
      {
        upload_id: uploadId,
        object_key: objectKey,
        filename: filename,
        parts: parts,
      },
      { headers: { Authorization: `Bearer ${accessToken}` } },
    );
    return response.data;
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function getFileById(
  accessToken: string,
  fileId: number,
): Promise<FileItem> {
  try {
    return await axios.get(`${BASE_URL}/api/files/${fileId}`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function downloadFile(
  accessToken: string,
  fileId: number,
  filename: string,
): Promise<void> {
  try {
    const response = await axios.get(
      `${BASE_URL}/api/files/${fileId}/download`,
      {
        headers: { Authorization: `Bearer ${accessToken}` },
        responseType: "blob",
      },
    );
    const url = URL.createObjectURL(response.data);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  } catch (err) {
    console.log(err);
    throw err;
  }
}

export async function uploadFile(
  accessToken: string,
  file: File,
): Promise<FileItem> {
  try {
    const CHUNK_SIZE = 1024 * 1024 * 10; //10 mbs
    const { upload_id: uploadId, object_key: objectKey } =
      await initiateUpload(accessToken);
    const parts: PartInfo[] = [];

    let partNumber = 1;
    for (let start = 0; start < file.size; start += CHUNK_SIZE) {
      const chunk = file.slice(start, start + CHUNK_SIZE);
      const part = await uploadChunk(
        accessToken,
        objectKey,
        uploadId,
        partNumber,
        chunk,
      );
      parts.push(part.part);
      partNumber++;
    }
    const fileItem = await completeUpload(
      accessToken,
      uploadId,
      objectKey,
      file.name,
      parts,
    );

    return fileItem;
  } catch (err) {
    console.log(err);
    throw err;
  }
}
