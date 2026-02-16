import { useState, useEffect } from 'react';
import { filesService } from '../services/filesService';
import type { FileMetadata } from '../services/filesService';

export function FileManager() {
  const [files, setFiles] = useState<FileMetadata[]>([]);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadFiles();
  }, []);

  const loadFiles = async () => {
    setLoading(true);
    const response = await filesService.listFiles();

    if (response.error) {
      setError(response.error);
    } else if (response.data) {
      setFiles(response.data);
    }

    setLoading(false);
  };

  const handleDelete = async (fileId: string) => {
    if (!confirm('Are you sure you want to delete this file?')) {
      return;
    }

    setError('');
    setSuccess('');

    const response = await filesService.deleteFile(fileId);

    if (response.error) {
      setError(response.error);
    } else {
      setSuccess('File deleted successfully!');
      setFiles(files.filter(f => f.id !== fileId));
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  return (
    <div className="file-manager">
      <h2>File Manager</h2>

      {error && <div className="error">{error}</div>}
      {success && <div className="success">{success}</div>}

      {loading ? (
        <p>Loading files...</p>
      ) : files.length === 0 ? (
        <p className="no-files">No files uploaded yet.</p>
      ) : (
        <div className="files-list">
          {files.map((file) => (
            <div key={file.id} className="file-item">
              <div className="file-info">
                <h3>{file.filename}</h3>
                <p>Size: {formatFileSize(file.size)}</p>
                <p>Type: {file.content_type}</p>
                <p>Uploaded: {formatDate(file.created_at)}</p>
              </div>
              <button onClick={() => handleDelete(file.id)} className="delete-btn">
                Delete
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
