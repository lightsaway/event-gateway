import React, { useEffect, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ConditionBuilder } from './condition-builder';
import { TopicRoutingRule, Condition } from '@/services/routing-rules';

interface RuleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (rule: Omit<TopicRoutingRule, 'id'>) => Promise<void>;
  initialData?: TopicRoutingRule;
}

const defaultCondition: Condition = {
  type: 'equals',
  value: '',
};

export function RuleDialog({
  open,
  onOpenChange,
  onSave,
  initialData,
}: RuleDialogProps) {
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState<Omit<TopicRoutingRule, 'id'>>({
    order: 0,
    topic: '',
    description: '',
    eventTypeCondition: defaultCondition,
    eventVersionCondition: undefined,
  });

  useEffect(() => {
    if (initialData) {
      const { id, ...data } = initialData;
      setFormData(data);
    } else {
      setFormData({
        order: 0,
        topic: '',
        description: '',
        eventTypeCondition: defaultCondition,
        eventVersionCondition: undefined,
      });
    }
  }, [initialData, open]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      setLoading(true);
      await onSave(formData);
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to save rule:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>{initialData ? 'Edit Rule' : 'Add Rule'}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="order">Order</Label>
              <Input
                id="order"
                type="number"
                value={formData.order}
                onChange={(e) => setFormData({ ...formData, order: parseInt(e.target.value) })}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="topic">Topic</Label>
              <Input
                id="topic"
                value={formData.topic}
                onChange={(e) => setFormData({ ...formData, topic: e.target.value })}
                required
              />
            </div>
          </div>

          <div className="space-y-2">
            <Label>Event Type Condition</Label>
            <ConditionBuilder
              value={formData.eventTypeCondition}
              onChange={(condition) => setFormData({ ...formData, eventTypeCondition: condition })}
            />
          </div>

          <div className="space-y-2">
            <Label>Event Version Condition (Optional)</Label>
            <ConditionBuilder
              value={formData.eventVersionCondition || defaultCondition}
              onChange={(condition) => setFormData({ ...formData, eventVersionCondition: condition })}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description (Optional)</Label>
            <Input
              id="description"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            />
          </div>

          <div className="flex justify-end gap-2">
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
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
} 