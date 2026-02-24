import axios from "axios";

const BASE_URL = "http://localhost:3000";
axios.defaults.withCredentials = true;

export async function login(emailOrUsername: string, password: string) {
  //determine if the value is an email or a username
  if (emailOrUsername.includes("@")) {
    //email
    try {
      return await axios.post(`${BASE_URL}/api/auth/login`, {
        email: emailOrUsername,
        password,
      });
    } catch (err) {
      console.log("failed to login - " + err);
      throw err;
    }
  } else {
    //username
    try {
      return await axios.post(`${BASE_URL}/api/auth/login`, {
        user_name: emailOrUsername,
        password,
      });
    } catch (err) {
      console.log("failed to login - " + err);
      throw err;
    }
  }
}

export async function refresh() {
  try {
    return await axios.post(`${BASE_URL}/api/auth/refresh`);
  } catch (err) {
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
