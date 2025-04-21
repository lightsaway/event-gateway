import { Uuid } from './common';

export type DataType = 'string' | 'json' | 'binary';

export interface Event {
  id: string;
  type: string;
  version?: string;
  data: any;
  metadata?: Record<string, any>;
  transport_metadata?: Record<string, any>;
  timestamp: string;
  origin: string;
}

export interface EventFormValues {
  type: string;
  version?: string;
  data: string;
  dataType: DataType;
  metadata?: string;
  transportMetadata?: string;
} 