import axios from "axios";

const BASE_URL = "http://localhost:3000";

function authHeader(accessToken: string) {
  return { Authorization: `Bearer ${accessToken}` };
}

export async function getCurrentUser(accessToken: string) {
  return await axios.get(`${BASE_URL}/api/users/me`, {
    headers: authHeader(accessToken),
  });
}

export async function updateProfile(
  accessToken: string,
  data: {
    first_name: string;
    last_name: string;
    user_name: string;
    email: string;
    phone_number: string;
    is_male: boolean | null | undefined;
  }
) {
  return await axios.patch(`${BASE_URL}/api/users/me`, data, {
    headers: authHeader(accessToken),
  });
}

export async function getSessions(accessToken: string) {
  return await axios.get(`${BASE_URL}/api/auth/sessions`, {
    headers: authHeader(accessToken),
  });
}

export async function revokeSession(accessToken: string, sessionId: number) {
  return await axios.delete(`${BASE_URL}/api/auth/sessions/${sessionId}`, {
    headers: authHeader(accessToken),
  });
}
