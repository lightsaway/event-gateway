import { Uuid } from '../types/common';

export interface StringExpression {
  type: 'equals';
  value: string;
}

export interface Condition {
  type: string;
  value: string;
}

export interface TopicRoutingRule {
  id: Uuid;
  order: number;
  topic: string;
  description?: string;
  eventVersionCondition?: Condition;
  eventTypeCondition: Condition;
}

const API_BASE = '/api/v1';

export async function getAllRules(): Promise<TopicRoutingRule[]> {
  const response = await fetch(`${API_BASE}/routing-rules`);
  if (!response.ok) {
    throw new Error('Failed to fetch routing rules');
  }
  const data = await response.json();
  return Array.isArray(data) ? data : [];
}

export async function getRule(id: Uuid): Promise<TopicRoutingRule> {
  const response = await fetch(`${API_BASE}/routing-rules/${id}`);
  if (!response.ok) {
    throw new Error('Failed to fetch routing rule');
  }
  const data = await response.json();
  return data.rule;
}

export async function createRule(rule: Omit<TopicRoutingRule, 'id'>): Promise<TopicRoutingRule> {
  const response = await fetch(`${API_BASE}/routing-rules`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(rule),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to create routing rule' }));
    throw new Error(error.message || 'Failed to create routing rule');
  }
  const data = await response.json();
  return data.rule;
}

export async function updateRule(id: Uuid, rule: TopicRoutingRule): Promise<void> {
  const response = await fetch(`${API_BASE}/routing-rules/${id}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(rule),
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to update routing rule' }));
    throw new Error(error.message || 'Failed to update routing rule');
  }
}

export async function deleteRule(id: Uuid): Promise<void> {
  const response = await fetch(`${API_BASE}/routing-rules/${id}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Failed to delete routing rule' }));
    throw new Error(error.message || 'Failed to delete routing rule');
  }
} 