import { useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import DashboardPage from "./pages/DashboardPage";
import { useAuth } from "./context/AuthContext";
import { refresh } from "./api/auth";
import { getCurrentUser } from "./api/user";
import { tryCatch } from "./api/tryCatch";

function App() {
  const { setAccessToken, setUser } = useAuth();

  useEffect(() => {
    async function tryRestoreSession() {
      const [refreshData, refreshErr] = await tryCatch(refresh());
      if (refreshErr) return;

      const accessToken = refreshData?.data.access_token;
      setAccessToken(accessToken);

      const [userData, userErr] = await tryCatch(getCurrentUser(accessToken));
      if (userErr) return;

      setUser(userData?.data);
    }
    tryRestoreSession();
  }, []);

  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/" element={<DashboardPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
