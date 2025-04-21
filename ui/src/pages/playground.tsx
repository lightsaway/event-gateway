import React, { useState } from 'react';
import { Form, Input, Select, Button, message, Tabs, Table, Tag, Tooltip } from 'antd';
import { v4 as uuidv4 } from 'uuid';
import { Event, EventFormValues, DataType } from '../types/events';
import { sendEvent } from '../services/events';

const { TabPane } = Tabs;

interface StoredEvent extends Event {
  submittedAt: string;
  result: 'success' | 'error';
  errorMessage?: string;
}

const PlaygroundPage: React.FC = () => {
  const [form] = Form.useForm();
  const [rawJson, setRawJson] = useState('');
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<StoredEvent[]>([]);

  const saveEventToStorage = (event: Event, result: 'success' | 'error', errorMessage?: string) => {
    const storedEvent: StoredEvent = {
      ...event,
      submittedAt: new Date().toISOString(),
      result,
      errorMessage,
    };
    setHistory(prev => [storedEvent, ...prev]);
  };

  const handleSubmit = async (values: EventFormValues) => {
    setLoading(true);
    try {
      const event: Event = {
        id: uuidv4(),
        type: values.type,
        version: values.version,
        data: values.dataType === 'json' ? JSON.parse(values.data) : values.data,
        metadata: values.metadata ? JSON.parse(values.metadata) : undefined,
        transport_metadata: values.transportMetadata ? JSON.parse(values.transportMetadata) : undefined,
        timestamp: new Date().toISOString(),
        origin: 'playground',
      };

      await sendEvent(event);
      message.success('Event sent successfully');
      saveEventToStorage(event, 'success');
      form.resetFields();
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to send event';
      message.error(errorMessage);
      // Create a minimal event object for error case
      const errorEvent: Event = {
        id: uuidv4(),
        type: values.type,
        version: values.version,
        data: values.dataType === 'json' ? JSON.parse(values.data) : values.data,
        timestamp: new Date().toISOString(),
        origin: 'playground',
      };
      saveEventToStorage(errorEvent, 'error', errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleRawSubmit = async () => {
    setLoading(true);
    try {
      const event = JSON.parse(rawJson);
      await sendEvent(event);
      message.success('Event sent successfully');
      saveEventToStorage(event, 'success');
      setRawJson('');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to send event';
      message.error(errorMessage);
      try {
        const event = JSON.parse(rawJson);
        saveEventToStorage(event, 'error', errorMessage);
      } catch {
        // If we can't parse the JSON, don't save to history
      }
    } finally {
      setLoading(false);
    }
  };

  const columns = [
    {
      title: 'Type',
      dataIndex: 'type',
      key: 'type',
    },
    {
      title: 'Version',
      dataIndex: 'version',
      key: 'version',
    },
    {
      title: 'Submitted At',
      dataIndex: 'submittedAt',
      key: 'submittedAt',
      render: (date: string) => new Date(date).toLocaleString(),
    },
    {
      title: 'Result',
      dataIndex: 'result',
      key: 'result',
      render: (result: string, record: StoredEvent) => (
        <Tooltip title={record.errorMessage}>
          <Tag color={result === 'success' ? 'green' : 'red'}>
            {result === 'success' ? 'Success' : 'Failed'}
          </Tag>
        </Tooltip>
      ),
    },
  ];

  return (
    <div style={{ padding: '24px' }}>
      <Tabs defaultActiveKey="form">
        <TabPane tab="Send Event" key="form">
          <Form
            form={form}
            layout="vertical"
            onFinish={handleSubmit}
            style={{ maxWidth: 600 }}
          >
            <Form.Item
              name="type"
              label="Event Type"
              rules={[{ required: true, message: 'Please input event type!' }]}
            >
              <Input placeholder="e.g., user.created" />
            </Form.Item>

            <Form.Item
              name="version"
              label="Event Version"
            >
              <Input placeholder="e.g., 1.0" />
            </Form.Item>

            <Form.Item
              name="dataType"
              label="Data Type"
              initialValue="json"
            >
              <Select>
                <Select.Option value="json">JSON</Select.Option>
                <Select.Option value="string">String</Select.Option>
                <Select.Option value="binary">Binary</Select.Option>
              </Select>
            </Form.Item>

            <Form.Item
              name="data"
              label="Data"
              rules={[{ required: true, message: 'Please input event data!' }]}
            >
              <Input.TextArea rows={4} placeholder="Enter event data" />
            </Form.Item>

            <Form.Item
              name="metadata"
              label="Metadata (JSON)"
            >
              <Input.TextArea rows={2} placeholder="Enter metadata as JSON" />
            </Form.Item>

            <Form.Item
              name="transportMetadata"
              label="Transport Metadata (JSON)"
            >
              <Input.TextArea rows={2} placeholder="Enter transport metadata as JSON" />
            </Form.Item>

            <Form.Item>
              <Button 
                type="primary" 
                htmlType="submit" 
                loading={loading}
                style={{ 
                  backgroundColor: 'black',
                  width: '120px'
                }}
              >
                Send Event
              </Button>
            </Form.Item>
          </Form>
        </TabPane>

        <TabPane tab="Raw JSON" key="raw">
          <div style={{ maxWidth: 600 }}>
            <Input.TextArea
              value={rawJson}
              onChange={(e) => setRawJson(e.target.value)}
              rows={10}
              placeholder={`{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "type": "user.created",
  "version": "1.0",
  "data": {
    "userId": "123",
    "email": "user@example.com",
    "name": "John Doe"
  },
  "metadata": {
    "source": "web",
    "ip": "192.168.1.1"
  },
  "transport_metadata": {
    "topic": "users",
    "partition": 0
  },
  "timestamp": "2024-03-20T12:00:00Z",
  "origin": "playground"
}`}
            />
            <Button
              type="primary"
              onClick={handleRawSubmit}
              loading={loading}
              style={{ 
                marginTop: 16, 
                backgroundColor: 'black',
                width: '120px'
              }}
            >
              Send Event
            </Button>
          </div>
        </TabPane>

        <TabPane tab="History" key="history">
          <Table
            dataSource={history}
            columns={columns}
            rowKey="id"
            pagination={{ pageSize: 10 }}
          />
        </TabPane>
      </Tabs>
    </div>
  );
};

export default PlaygroundPage; 