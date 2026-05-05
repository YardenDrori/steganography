import { useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import DashboardPage from "./pages/DashboardPage";
import SettingsPage from "./pages/SettingsPage";
import EmbedPage from "./pages/EmbedPage";
import ExtractPage from "./pages/ExtractPage";
import FilesPage from "./pages/FilesPage";
import AdminPage from "./pages/AdminPage";
import { useAuth } from "./context/AuthContext";
import { extractIsAdmin } from "./context/AuthContext";
import { refresh } from "./api/auth";
import { getCurrentUser } from "./api/user";
import { tryCatch } from "./api/tryCatch";
import { Layout } from "./components/Layout";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();
  if (isLoading) return null;
  if (!user) return <Navigate to="/login" />;
  return <Layout>{children}</Layout>;
}

function AdminRoute({ children }: { children: React.ReactNode }) {
  const { user, isLoading } = useAuth();
  if (isLoading) return null;
  if (!user?.is_admin) return <Navigate to="/" />;
  return <Layout>{children}</Layout>;
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
      if (!userErr && userData?.data) {
        setUser({ ...userData.data, is_admin: extractIsAdmin(accessToken) });
      }

      setIsLoading(false);
    }
    tryRestoreSession();
  }, [setAccessToken, setIsLoading, setUser]);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<ProtectedRoute><DashboardPage /></ProtectedRoute>} />
        <Route path="/my-files" element={<ProtectedRoute><FilesPage /></ProtectedRoute>} />
        <Route path="/embed" element={<ProtectedRoute><EmbedPage /></ProtectedRoute>} />
        <Route path="/extract" element={<ProtectedRoute><ExtractPage /></ProtectedRoute>} />
        <Route path="/settings" element={<ProtectedRoute><SettingsPage /></ProtectedRoute>} />
        <Route path="/admin" element={<AdminRoute><AdminPage /></AdminRoute>} />
        {/* keep old route working */}
        <Route path="/my-files/embed" element={<Navigate to="/embed" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
