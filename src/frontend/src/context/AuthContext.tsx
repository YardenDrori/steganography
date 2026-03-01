import { createContext, useState, useContext } from "react";

type User = {
  id: number;
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  phone_number: string | null;
  is_male: boolean | null;
  is_verified: boolean;
  created_at: string;
  updated_at: string;
};

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
