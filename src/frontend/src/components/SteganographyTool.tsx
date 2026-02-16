import { useState } from 'react';
import { steganographyService } from '../services/steganographyService';

export function SteganographyTool() {
  const [imageFile, setImageFile] = useState<File | null>(null);
  const [message, setMessage] = useState('');
  const [preview, setPreview] = useState<string | null>(null);
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setImageFile(file);
      const reader = new FileReader();
      reader.onloadend = () => {
        setPreview(reader.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!imageFile) {
      setError('Please select an image');
      return;
    }

    if (!message.trim()) {
      setError('Please enter a message to embed');
      return;
    }

    setError('');
    setResult('');
    setLoading(true);

    const response = await steganographyService.embedImage(imageFile, message);

    if (response.error) {
      setError(response.error);
    } else if (response.data) {
      setResult(response.data.message);
      setMessage('');
      setImageFile(null);
      setPreview(null);
    }

    setLoading(false);
  };

  return (
    <div className="steganography-tool">
      <h2>Embed Message in Image</h2>

      {error && <div className="error">{error}</div>}
      {result && <div className="success">{result}</div>}

      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label htmlFor="image">Select Image</label>
          <input
            id="image"
            type="file"
            accept="image/*"
            onChange={handleFileChange}
            disabled={loading}
          />
        </div>

        {preview && (
          <div className="image-preview">
            <img src={preview} alt="Preview" />
          </div>
        )}

        <div className="form-group">
          <label htmlFor="message">Secret Message</label>
          <textarea
            id="message"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder="Enter your secret message..."
            rows={4}
            disabled={loading}
          />
        </div>

        <button type="submit" disabled={loading || !imageFile || !message.trim()}>
          {loading ? 'Embedding...' : 'Embed Message'}
        </button>
      </form>
    </div>
  );
}
