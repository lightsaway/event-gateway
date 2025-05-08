export type DataType = 'string' | 'json' | 'binary';

export interface Event {
  id: string;
  eventType: string;
  eventVersion?: string;
  data: any;
  metadata?: Record<string, any>;
  transportMetadata?: Record<string, any>;
  dataType?: DataType;
  timestamp?: string;
  origin?: string;
}

export interface EventFormValues {
  type: string;
  version?: string;
  data: string;
  dataType: DataType;
  metadata?: string;
  transportMetadata?: string;
} 