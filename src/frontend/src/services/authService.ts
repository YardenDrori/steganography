import { apiRequest } from './api';

export interface RegisterData {
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  password: string;
  is_male: boolean;
}

export interface LoginData {
  email?: string;
  user_name?: string;
  password: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  user: {
    id: number;
    user_name: string;
    first_name: string;
    last_name: string;
    email: string;
    is_male: boolean;
  };
}

export const authService = {
  async register(data: RegisterData) {
    return apiRequest<AuthResponse>('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async login(data: LoginData) {
    return apiRequest<AuthResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async refresh(refreshToken: string) {
    return apiRequest<{ access_token: string; refresh_token: string }>('/api/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken }),
    });
  },

  async deactivate() {
    return apiRequest<{ message: string }>('/api/auth/deactivate', {
      method: 'POST',
    });
  },
};
