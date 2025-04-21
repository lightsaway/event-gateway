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
import { TopicRoutingRule } from '@/services/routing-rules';

interface RuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (rule: Omit<TopicRoutingRule, 'id'>) => Promise<void>;
  initialData?: TopicRoutingRule;
}

export function RuleDialog({ open, onOpenChange, onSave, initialData }: RuleDialogProps) {
  const [formData, setFormData] = useState({
    order: initialData?.order ?? 0,
    topic: initialData?.topic ?? '',
    description: initialData?.description ?? '',
    eventTypeCondition: {
      type: 'equals',
      value: initialData?.eventTypeCondition.value ?? '',
    },
    eventVersionCondition: initialData?.eventVersionCondition
      ? {
          type: 'equals',
          value: initialData.eventVersionCondition.value,
        }
      : undefined,
  });

  useEffect(() => {
    if (initialData) {
      setFormData({
        order: initialData.order,
        topic: initialData.topic,
        description: initialData.description ?? '',
        eventTypeCondition: {
          type: 'equals',
          value: initialData.eventTypeCondition.value,
        },
        eventVersionCondition: initialData.eventVersionCondition
          ? {
              type: 'equals',
              value: initialData.eventVersionCondition.value,
            }
          : undefined,
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
      await onSave(formData);
      onOpenChange(false);
    } catch (err) {
      setError('Failed to save rule');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{initialData ? 'Edit Rule' : 'Add Rule'}</DialogTitle>
          <DialogDescription>
            {initialData
              ? 'Edit the routing rule details below.'
              : 'Create a new routing rule by filling out the form below.'}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="order" className="text-right">
                Order
              </Label>
              <Input
                id="order"
                type="number"
                value={formData.order}
                onChange={(e) =>
                  setFormData({ ...formData, order: parseInt(e.target.value, 10) })
                }
                className="col-span-3"
              />
            </div>

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
              <Label htmlFor="eventType" className="text-right">
                Event Type
              </Label>
              <Input
                id="eventType"
                value={formData.eventTypeCondition.value}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    eventTypeCondition: {
                      type: 'equals',
                      value: e.target.value,
                    },
                  })
                }
                className="col-span-3"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="eventVersion" className="text-right">
                Event Version
              </Label>
              <Input
                id="eventVersion"
                value={formData.eventVersionCondition?.value ?? ''}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    eventVersionCondition: e.target.value
                      ? {
                          type: 'equals',
                          value: e.target.value,
                        }
                      : undefined,
                  })
                }
                placeholder="Optional"
                className="col-span-3"
              />
            </div>

            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">
                Description
              </Label>
              <Input
                id="description"
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                placeholder="Optional"
                className="col-span-3"
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