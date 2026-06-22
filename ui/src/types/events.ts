export type DataType = 'string' | 'json' | 'binary';

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

export interface Event {
  id: string;
  eventType: string;
  eventVersion?: string;
  data: JsonValue;
  metadata?: Record<string, JsonValue>;
  transportMetadata?: Record<string, JsonValue>;
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
