import axios from "axios";

const BASE_URL = "http://localhost:3000";

function authHeader(token: string) {
  return { Authorization: `Bearer ${token}` };
}

export async function listAllUsers(token: string) {
  return axios.get(`${BASE_URL}/api/users/all`, { headers: authHeader(token) });
}

export async function listAllFiles(token: string) {
  return axios.get(`${BASE_URL}/api/files/all`, { headers: authHeader(token) });
}

export async function activateUser(token: string, id: number) {
  return axios.patch(`${BASE_URL}/api/auth/admin/users/${id}/activate`, {}, { headers: authHeader(token) });
}

export async function deactivateUser(token: string, id: number) {
  return axios.patch(`${BASE_URL}/api/auth/admin/users/${id}/deactivate`, {}, { headers: authHeader(token) });
}

export async function adminDeleteUser(token: string, id: number) {
  return axios.delete(`${BASE_URL}/api/users/${id}`, { headers: authHeader(token) });
}

export async function adminDeleteFile(token: string, id: number) {
  return axios.delete(`${BASE_URL}/api/files/${id}`, { headers: authHeader(token) });
}
