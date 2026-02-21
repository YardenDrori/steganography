import { useState } from "react";
import { login } from "../api/auth";
import { tryCatch } from "../api/tryCatch";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const auth = useAuth();
  const navigator = useNavigate();

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    const [data, caughtError] = await tryCatch(login(email, password));
    if (caughtError) {
      setError(caughtError);
    } else {
      console.log(data);
      auth.setAccessToken(data?.data.access_token);
      auth.setUser(data?.data.user);
      navigator("/");
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      <input
        type="email"
        placeholder="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
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
  );
}

export default LoginPage;
