import axios from "axios";

const BASE_URL = "http://localhost:3000";

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
    throw err;
  }
}
