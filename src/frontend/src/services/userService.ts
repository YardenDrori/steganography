import { apiRequest } from './api';

export interface User {
  id: number;
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  is_male: boolean;
}

export interface UpdateUserData {
  first_name?: string;
  last_name?: string;
}

export const userService = {
  async getMe() {
    return apiRequest<User>('/api/users/me', {
      method: 'GET',
    });
  },

  async updateMe(data: UpdateUserData) {
    return apiRequest<User>('/api/users/me', {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  },
};
