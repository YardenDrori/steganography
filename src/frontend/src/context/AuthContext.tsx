import { createContext, useState, useContext, useEffect } from "react";
import { registerSessionHandlers } from "../api/authInterceptor";

export type User = {
  id: number;
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  phone_number: string | null;
  is_male: boolean | null;
  is_active: boolean;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
};

export function extractIsAdmin(token: string): boolean {
  try {
    const payload = JSON.parse(atob(token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/")));
    return Array.isArray(payload.roles) && payload.roles.includes("admin");
  } catch {
    return false;
  }
}

type AuthContextType = {
  accessToken: string | null;
  user: User | null;
  isLoading: boolean;
  setAccessToken: (token: string | null) => void;
  setUser: (user: User | null) => void;
  setIsLoading: (value: boolean) => void;
};

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider(props: { children: React.ReactNode }) {
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    registerSessionHandlers(
      (newToken) => setAccessToken(newToken),
      () => { setAccessToken(null); setUser(null); },
    );
  }, []);

  return (
    <AuthContext.Provider
      value={{
        accessToken,
        user,
        isLoading,
        setAccessToken,
        setUser,
        setIsLoading,
      }}
    >
      {props.children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  return useContext(AuthContext)!;
}
