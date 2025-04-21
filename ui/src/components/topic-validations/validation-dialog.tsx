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
import { TopicValidationConfig, DataSchema } from '@/services/topic-validations';

interface ValidationDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (validation: Omit<TopicValidationConfig, 'id'>) => Promise<void>;
  initialData?: TopicValidationConfig;
}

export function ValidationDialog({ open, onOpenChange, onSave, initialData }: ValidationDialogProps) {
  const [formData, setFormData] = useState({
    topic: initialData?.topic ?? '',
    schemas: initialData?.schemas ?? [],
  });

  const [newSchema, setNewSchema] = useState({
    name: '',
    schema: '',
    description: '',
  });

  useEffect(() => {
    if (initialData) {
      setFormData({
        topic: initialData.topic,
        schemas: initialData.schemas,
      });
    }
  }, [initialData]);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function handleAddSchema() {
    if (!newSchema.name || !newSchema.schema) {
      return;
    }

    setFormData({
      ...formData,
      schemas: [
        ...formData.schemas,
        {
          id: crypto.randomUUID(),
          name: newSchema.name,
          schema: newSchema.schema,
          description: newSchema.description || undefined,
        },
      ],
    });

    setNewSchema({
      name: '',
      schema: '',
      description: '',
    });
  }

  function handleRemoveSchema(id: string) {
    setFormData({
      ...formData,
      schemas: formData.schemas.filter((schema) => schema.id !== id),
    });
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      setLoading(true);
      setError(null);
      await onSave(formData);
      onOpenChange(false);
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

            <div className="col-span-4">
              <h3 className="text-lg font-medium mb-2">Schemas</h3>
              <div className="space-y-4">
                {formData.schemas.map((schema) => (
                  <div
                    key={schema.id}
                    className="p-4 border rounded-lg space-y-2"
                  >
                    <div className="flex justify-between items-start">
                      <div>
                        <h4 className="font-medium">{schema.name}</h4>
                        {schema.description && (
                          <p className="text-sm text-muted-foreground">
                            {schema.description}
                          </p>
                        )}
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => handleRemoveSchema(schema.id)}
                      >
                        Remove
                      </Button>
                    </div>
                    <pre className="bg-muted p-2 rounded text-sm overflow-x-auto">
                      {schema.schema}
                    </pre>
                  </div>
                ))}

                <div className="border rounded-lg p-4 space-y-4">
                  <h4 className="font-medium">Add New Schema</h4>
                  <div className="grid gap-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                      <Label htmlFor="schemaName" className="text-right">
                        Name
                      </Label>
                      <Input
                        id="schemaName"
                        value={newSchema.name}
                        onChange={(e) =>
                          setNewSchema({ ...newSchema, name: e.target.value })
                        }
                        className="col-span-3"
                      />
                    </div>

                    <div className="grid grid-cols-4 items-center gap-4">
                      <Label htmlFor="schemaDescription" className="text-right">
                        Description
                      </Label>
                      <Input
                        id="schemaDescription"
                        value={newSchema.description}
                        onChange={(e) =>
                          setNewSchema({
                            ...newSchema,
                            description: e.target.value,
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
                        value={newSchema.schema}
                        onChange={(e) =>
                          setNewSchema({ ...newSchema, schema: e.target.value })
                        }
                        className="col-span-3 h-32 font-mono"
                        placeholder="Enter JSON schema"
                      />
                    </div>

                    <div className="col-span-4 flex justify-end">
                      <Button
                        type="button"
                        onClick={handleAddSchema}
                        disabled={!newSchema.name || !newSchema.schema}
                      >
                        Add Schema
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
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