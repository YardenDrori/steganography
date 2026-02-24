import { useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import DashboardPage from "./pages/DashboardPage";
import SettingsPage from "./pages/SettingsPage";
import { useAuth } from "./context/AuthContext";
import { refresh } from "./api/auth";
import { getCurrentUser } from "./api/user";
import { tryCatch } from "./api/tryCatch";

function ProtectedRoute(props: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();
  if (isLoading) return null;
  if (!user) return <Navigate to={"/login"} />;
  return props.children;
}

function App() {
  const { setAccessToken, setUser, setIsLoading } = useAuth();

  useEffect(() => {
    async function tryRestoreSession() {
      const [refreshData, refreshErr] = await tryCatch(refresh());
      if (refreshErr) {
        setIsLoading(false);
        return;
      }

      const accessToken = refreshData?.data.access_token;
      setAccessToken(accessToken);

      const [userData, userErr] = await tryCatch(getCurrentUser(accessToken));
      if (!userErr) {
        setUser(userData?.data);
      }

      setIsLoading(false);
    }
    tryRestoreSession();
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <DashboardPage />
            </ProtectedRoute>
          }
        />
        <Route
          path="/settings"
          element={
            <ProtectedRoute>
              <SettingsPage />
            </ProtectedRoute>
          }
        />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
