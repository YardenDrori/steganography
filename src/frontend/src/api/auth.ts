import axios from "axios";

const BASE_URL = "http://localhost:3000";
axios.defaults.withCredentials = true;

export async function login(email: string, password: string) {
  try {
    return await axios.post(`${BASE_URL}/api/auth/login`, {
      email,
      password,
    });
  } catch (err) {
    console.log("failed to login - " + err);
    throw err;
  }
}

export async function logout() {
  try {
    return await axios.post(`${BASE_URL}/api/auth/logout`);
  } catch (err) {
    console.log("failed to logout - " + err);
    throw err;
  }
}

export async function register(form: {
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  phone_number?: string;
  is_male?: boolean;
  password: string;
}) {
  try {
    return await axios.post(`${BASE_URL}/api/auth/register`, form);
  } catch (err) {
    console.log("failed to signup - " + err);
    if (axios.isAxiosError(err)) {
      console.log(err.response?.data);
    }
    throw err;
  }
}
