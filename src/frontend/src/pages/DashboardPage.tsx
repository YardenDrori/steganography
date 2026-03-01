import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";
import { logout } from "../api/auth";
import { tryCatch } from "../api/tryCatch";

function DashboardPage() {
  const { user, setAccessToken, setUser } = useAuth();
  const navigate = useNavigate();

  async function handleLogout() {
    const [, err] = await tryCatch(logout());
    if (err) {
      console.log("Logout failed: " + err);
    }
    setAccessToken(null);
    setUser(null);
    navigate("/login");
  }

  return (
    <div>
      <h1>Dashboard</h1>
      <p>
        Welcome, {user?.first_name} {user?.last_name}
      </p>
      <p>Username: {user?.user_name}</p>
      <p>Email: {user?.email}</p>
      <button type="button" onClick={() => navigate("/settings")}>
        Settings
      </button>
      <button onClick={handleLogout}>Logout</button>
    </div>
  );
}

export default DashboardPage;
