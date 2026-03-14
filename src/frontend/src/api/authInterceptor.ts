import axios from "axios";
import { refresh } from "./auth";

let onRefreshed: ((token: string) => void) | null = null;
let onExpired: (() => void) | null = null;

// Called once from AuthContext on mount
export function registerSessionHandlers(
  refreshedCallback: (token: string) => void,
  expiredCallback: () => void,
) {
  onRefreshed = refreshedCallback;
  onExpired = expiredCallback;
}

// Shared promise so concurrent 401s all wait on the same refresh call
// instead of firing multiple refresh requests
let refreshPromise: Promise<string> | null = null;

axios.interceptors.response.use(
  (res) => res,
  async (error) => {
    const config = error.config;

    // If the refresh call itself failed → session is fully expired
    if (config?.url?.includes("/auth/refresh")) {
      onExpired?.();
      return Promise.reject(error);
    }

    // Only act on 401s we haven't already retried
    if (error.response?.status !== 401 || config?._retry) {
      return Promise.reject(error);
    }

    config._retry = true;

    try {
      if (!refreshPromise) {
        refreshPromise = refresh()
          .then((res) => {
            const token: string = res.data.access_token;
            onRefreshed?.(token);
            return token;
          })
          .finally(() => {
            refreshPromise = null;
          });
      }

      const newToken = await refreshPromise;
      config.headers["Authorization"] = `Bearer ${newToken}`;
      return axios(config);
    } catch {
      onExpired?.();
      return Promise.reject(error);
    }
  },
);
