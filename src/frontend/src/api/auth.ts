import axios from "axios";

const BASE_URL = "http://localhost:3000";

export async function login(email: string, password: string) {
  const response = await axios.post(`${BASE_URL}/api/auth/login`, {
    email,
    password,
  });
  return response.data;
}
