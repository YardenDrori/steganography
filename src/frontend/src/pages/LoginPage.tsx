import { useState } from "react";
import { login } from "../api/auth";
import { tryCatch } from "../api/tryCatch";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

function LoginPage() {
  const [emailOrUsername, setEmailOrUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const auth = useAuth();
  const navigate = useNavigate();

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    const [data, caughtError] = await tryCatch(login(emailOrUsername, password));
    if (caughtError) {
      setError(caughtError);
    } else {
      console.log(data);
      auth.setAccessToken(data?.data.access_token);
      auth.setUser(data?.data.user);
      navigate("/");
    }
  }

  return (
    <>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          placeholder="Email or Username"
          value={emailOrUsername}
          onChange={(e) => setEmailOrUsername(e.target.value)}
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        {error && <p style={{ color: "red" }}>{error}</p>}
        <button type="submit">Login</button>
      </form>
      <button type="button" onClick={() => navigate("/register")}>
        Sign Up
      </button>
    </>
  );
}

export default LoginPage;
