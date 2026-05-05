import axios from "axios";

export type AdminUser = {
  id: number;
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  phone_number: string | null;
  is_male: boolean | null;
  is_active: boolean;
  created_at: string;
  updated_at: string;
};

export type AdminFile = {
  id: number;
  user_id: number;
  filename: string;
  created_at: string;
  is_carrier: boolean;
  is_steg_object: boolean;
};

export type ServiceHealth = {
  name: string;
  ok: boolean;
};

function authHeader(token: string) {
  return { Authorization: `Bearer ${token}` };
}

export const getAllUsers = (token: string) =>
  axios.get<AdminUser[]>("/api/users/admin/all", { headers: authHeader(token) });

export const getAllFiles = (token: string) =>
  axios.get<AdminFile[]>("/api/files/admin/all", { headers: authHeader(token) });

export const activateUser = (token: string, id: number) =>
  axios.patch(`/api/auth/admin/users/${id}/activate`, {}, { headers: authHeader(token) });

export const deactivateUser = (token: string, id: number) =>
  axios.patch(`/api/auth/admin/users/${id}/deactivate`, {}, { headers: authHeader(token) });

export const deleteUserAdmin = (token: string, id: number) =>
  axios.delete(`/api/users/${id}`, { headers: authHeader(token) });

export const checkServiceHealth = async (token: string): Promise<ServiceHealth[]> => {
  const checks: { name: string; fn: () => Promise<unknown> }[] = [
    { name: "Auth", fn: () => axios.get("/api/auth/public-key") },
    { name: "Users", fn: () => axios.get("/api/users/me", { headers: authHeader(token) }) },
    { name: "Files", fn: () => axios.get("/api/files/me", { headers: authHeader(token) }) },
  ];

  return Promise.all(
    checks.map(async ({ name, fn }) => {
      try {
        await fn();
        return { name, ok: true };
      } catch {
        return { name, ok: false };
      }
    })
  );
};
