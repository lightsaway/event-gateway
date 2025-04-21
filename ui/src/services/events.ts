import { fetchApi } from './api';
import { Event } from '../types/events';

export async function sendEvent(event: Event): Promise<void> {
  await fetchApi('/event', {
    method: 'POST',
    body: JSON.stringify(event),
  });
} 