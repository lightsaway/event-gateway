import { Uuid } from '../types/common';

export interface DataSchema {
  id: Uuid;
  name: string;
  schema: string;
  description?: string;
  event_type: string;
  event_version: string;
}

export interface TopicValidationConfig {
  id: Uuid;
  topic: string;
  schema: DataSchema;
}

const API_BASE = '/api/v1';

export async function getAllValidations(): Promise<Record<string, TopicValidationConfig[]>> {
  const response = await fetch(`${API_BASE}/topic-validations`);
  if (!response.ok) {
    throw new Error('Failed to fetch topic validations');
  }
  return response.json();
}

export async function getValidation(id: Uuid): Promise<TopicValidationConfig> {
  const response = await fetch(`${API_BASE}/topic-validations/${id}`);
  if (!response.ok) {
    throw new Error('Failed to fetch topic validation');
  }
  const data = await response.json();
  return data.validation;
}

export async function createValidation(validation: Omit<TopicValidationConfig, 'id'>): Promise<void> {
  const response = await fetch(`${API_BASE}/topic-validations`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(validation),
  });
  if (!response.ok) {
    throw new Error('Failed to create topic validation');
  }
}

export async function updateValidation(id: Uuid, validation: TopicValidationConfig): Promise<void> {
  const response = await fetch(`${API_BASE}/topic-validations/${id}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(validation),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to update topic validation' }));
    throw new Error(error.message || 'Failed to update topic validation');
  }
}

export async function deleteValidation(id: Uuid): Promise<void> {
  const response = await fetch(`${API_BASE}/topic-validations/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error('Failed to delete topic validation');
  }
} 