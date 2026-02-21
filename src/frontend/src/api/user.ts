import axios from "axios";

const BASE_URL = "http://localhost:3000";

export async function getCurrentUser(accessToken: string) {
  try {
    return await axios.get(`${BASE_URL}/api/user/users/me`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
  } catch (err) {
    throw err;
  }
}
