import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { TopicValidationConfig } from '@/services/topic-validations';

interface ValidationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (validation: Omit<TopicValidationConfig, 'id'>) => Promise<void>;
  initialData?: TopicValidationConfig;
  onSuccess?: () => void;
}

export function ValidationDialog({ open, onOpenChange, onSave, initialData, onSuccess }: ValidationDialogProps) {
  const [formData, setFormData] = useState({
    topic: initialData?.topic ?? '',
    schema: {
      name: initialData?.schema.name ?? '',
      schema: initialData?.schema.schema ?? '',
      description: initialData?.schema.description ?? '',
      event_type: initialData?.schema.event_type ?? '',
      event_version: initialData?.schema.event_version ?? '',
    },
  });

  useEffect(() => {
    if (initialData) {
      setFormData({
        topic: initialData.topic,
        schema: {
          name: initialData.schema.name,
          schema: initialData.schema.schema,
          description: initialData.schema.description ?? '',
          event_type: initialData.schema.event_type,
          event_version: initialData.schema.event_version,
        },
      });
    }
  }, [initialData]);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      setLoading(true);
      setError(null);
      
      // Format the schema correctly to match the add_topic_validations.sh script
      const formattedSchema = {
        topic: formData.topic,
        schema: {
          id: initialData?.schema.id ?? crypto.randomUUID(),
          name: formData.schema.name,
          description: formData.schema.description || undefined,
          schema: {
            type: 'json',
            data: JSON.parse(formData.schema.schema)
          },
          event_type: formData.schema.event_type,
          event_version: formData.schema.event_version,
          metadata: {}
        },
      };
      
      await onSave(formattedSchema);
      onOpenChange(false);
      onSuccess?.();
    } catch (err) {
      setError('Failed to save validation');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>{initialData ? 'Edit Validation' : 'Add Validation'}</DialogTitle>
          <DialogDescription>
            {initialData
              ? 'Edit the topic validation details below.'
              : 'Create a new topic validation by filling out the form below.'}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="topic" className="text-right">
                Topic
              </Label>
              <Input
                id="topic"
                value={formData.topic}
                onChange={(e) =>
                  setFormData({ ...formData, topic: e.target.value })
                }
                className="col-span-3"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="schemaName" className="text-right">
                Schema Name
              </Label>
              <Input
                id="schemaName"
                value={formData.schema.name}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    schema: { ...formData.schema, name: e.target.value },
                  })
                }
                className="col-span-3"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="eventType" className="text-right">
                Event Type
              </Label>
              <Input
                id="eventType"
                value={formData.schema.event_type}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    schema: { ...formData.schema, event_type: e.target.value },
                  })
                }
                className="col-span-3"
                placeholder="e.g., user.created"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="eventVersion" className="text-right">
                Event Version
              </Label>
              <Input
                id="eventVersion"
                value={formData.schema.event_version}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    schema: { ...formData.schema, event_version: e.target.value },
                  })
                }
                className="col-span-3"
                placeholder="e.g., 1.0"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="schemaDescription" className="text-right">
                Description
              </Label>
              <Input
                id="schemaDescription"
                value={formData.schema.description}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    schema: { ...formData.schema, description: e.target.value },
                  })
                }
                placeholder="Optional"
                className="col-span-3"
              />
            </div>

            <div className="grid grid-cols-4 items-start gap-4">
              <Label htmlFor="schemaContent" className="text-right pt-2">
                Schema
              </Label>
              <Textarea
                id="schemaContent"
                value={formData.schema.schema}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    schema: { ...formData.schema, schema: e.target.value },
                  })
                }
                className="col-span-3 h-32 font-mono"
                placeholder="Enter JSON schema"
              />
            </div>
          </div>

          {error && <p className="text-sm text-red-500 mb-4">{error}</p>}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? 'Saving...' : 'Save'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
} 