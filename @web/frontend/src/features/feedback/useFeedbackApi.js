import api from '../../api/axios';

export async function submitReview({ rating, comment }) {
  const res = await api.post('/v1/feedback/reviews', { rating, comment });
  return res.data;
}

export async function submitBug({ description, logFile }) {
  const formData = new FormData();
  formData.append('description', description);
  if (logFile) formData.append('logFile', logFile);
  const res = await api.post('/v1/feedback/bug-reports', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  });
  return res.data;
}

export async function submitSurvey(data) {
  const res = await api.post('/v1/feedback/hardware-surveys', data);
  return res.data;
} 