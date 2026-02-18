function RegisterPage() {
  return (
    <form>
      <p>values with '*' indicate mandatory field</p>
      <input type="text" placeholder="Username*" />
      <input type="text" placeholder="First name*" />
      <input type="text" placeholder="Last name*" />
      <input type="email" placeholder="Email address*" />
      <input type="tel" placeholder="phone number" />
      <select>
        <option value="">Select gender</option>
        <option value="true">Male</option>
        <option value="false">Female</option>
      </select>
      <input type="password" placeholder="password*" />
      <input type="password" placeholder="confirm password*" />
      <button type="submit">Sign up</button>
    </form>
  );
}

export default RegisterPage;
