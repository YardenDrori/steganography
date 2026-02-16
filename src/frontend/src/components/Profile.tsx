import { useState } from 'react';
import { useAuth } from '../contexts/AuthContext';
import { userService } from '../services/userService';

export function Profile() {
  const { user, updateUser, logout } = useAuth();
  const [editing, setEditing] = useState(false);
  const [formData, setFormData] = useState({
    first_name: user?.first_name || '',
    last_name: user?.last_name || '',
  });
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setSuccess('');
    setLoading(true);

    const response = await userService.updateMe(formData);

    if (response.error) {
      setError(response.error);
    } else if (response.data) {
      updateUser(response.data);
      setSuccess('Profile updated successfully!');
      setEditing(false);
    }

    setLoading(false);
  };

  if (!user) return null;

  return (
    <div className="profile">
      <h2>Profile</h2>

      {error && <div className="error">{error}</div>}
      {success && <div className="success">{success}</div>}

      {!editing ? (
        <div className="profile-view">
          <p><strong>Username:</strong> {user.user_name}</p>
          <p><strong>Email:</strong> {user.email}</p>
          <p><strong>Name:</strong> {user.first_name} {user.last_name}</p>
          <p><strong>Gender:</strong> {user.is_male ? 'Male' : 'Female'}</p>

          <div className="button-group">
            <button onClick={() => setEditing(true)}>Edit Profile</button>
            <button onClick={logout} className="secondary">Logout</button>
          </div>
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="profile-form">
          <div className="form-group">
            <label htmlFor="first_name">First Name</label>
            <input
              id="first_name"
              type="text"
              value={formData.first_name}
              onChange={(e) => setFormData(prev => ({ ...prev, first_name: e.target.value }))}
              required
              disabled={loading}
            />
          </div>

          <div className="form-group">
            <label htmlFor="last_name">Last Name</label>
            <input
              id="last_name"
              type="text"
              value={formData.last_name}
              onChange={(e) => setFormData(prev => ({ ...prev, last_name: e.target.value }))}
              required
              disabled={loading}
            />
          </div>

          <div className="button-group">
            <button type="submit" disabled={loading}>
              {loading ? 'Saving...' : 'Save'}
            </button>
            <button type="button" onClick={() => setEditing(false)} className="secondary">
              Cancel
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
