import axios from "axios";

const BASE_URL = "http://localhost:3000";
axios.defaults.withCredentials = true;

export async function login(emailOrUsername: string, password: string) {
  const device_info = navigator.userAgent;
  const identity = emailOrUsername.includes("@")
    ? { email: emailOrUsername }
    : { user_name: emailOrUsername };
  try {
    return await axios.post(`${BASE_URL}/api/auth/login`, {
      ...identity,
      password,
      device_info,
    });
  } catch (err) {
    console.log("failed to login - " + err);
    throw err;
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

export async function changePassword(
  accessToken: string,
  old_password: string,
  new_password: string
) {
  try {
    return await axios.post(
      `${BASE_URL}/api/auth/change-password`,
      { old_password, new_password },
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
  } catch (err) {
    throw err;
  }
}

export async function logoutAllDevices(accessToken: string) {
  try {
    return await axios.post(
      `${BASE_URL}/api/auth/logout-all`,
      {},
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
  } catch (err) {
    throw err;
  }
}

export async function sendVerificationEmail(accessToken: string) {
  try {
    return await axios.post(
      `${BASE_URL}/api/auth/send-verification`,
      {},
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
  } catch (err) {
    throw err;
  }
}

export async function deleteAccount(accessToken: string) {
  try {
    return await axios.post(
      `${BASE_URL}/api/auth/deactivate`,
      {},
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
  } catch (err) {
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
