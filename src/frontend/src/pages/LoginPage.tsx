import { useState } from "react";
import { login } from "../api/auth";
import { tryCatch } from "../api/tryCatch";
import { useAuth } from "../context/AuthContext";

function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    const [data, caughtError] = await tryCatch(login(email, password));
    if (caughtError) {
      setError(caughtError);
    } else {
      const auth = useAuth();
      auth.setAccessToken(data?.data.access_token);
      console.log(data);
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
