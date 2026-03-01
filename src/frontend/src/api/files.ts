import axios from "axios";
import { tryCatch } from "./tryCatch";

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
