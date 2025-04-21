import React, { useEffect, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ConditionBuilder } from './condition-builder';
import { TopicRoutingRule, Condition } from '@/services/routing-rules';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

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
      <DialogContent className="sm:max-w-[700px]">
        <DialogHeader>
          <DialogTitle>{initialData ? 'Edit Rule' : 'Add Rule'}</DialogTitle>
          <DialogDescription>
            {initialData 
              ? 'Modify the routing rule details below.' 
              : 'Create a new routing rule by filling in the details below.'}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Basic Information</CardTitle>
              <CardDescription>Set the order and topic for this routing rule</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="order">Order</Label>
                  <Input
                    id="order"
                    type="number"
                    value={formData.order}
                    onChange={(e) => setFormData({ ...formData, order: parseInt(e.target.value) })}
                    required
                    min={0}
                  />
                  <p className="text-xs text-muted-foreground">Lower numbers have higher priority</p>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="topic">Topic</Label>
                  <Input
                    id="topic"
                    value={formData.topic}
                    onChange={(e) => setFormData({ ...formData, topic: e.target.value })}
                    required
                    placeholder="e.g. user.created"
                  />
                  <p className="text-xs text-muted-foreground">The topic to route matching events to</p>
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="description">Description (Optional)</Label>
                <Input
                  id="description"
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  placeholder="Brief description of this rule's purpose"
                />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Event Type Condition</CardTitle>
              <CardDescription>Define conditions for matching event types</CardDescription>
            </CardHeader>
            <CardContent>
              <ConditionBuilder
                value={formData.eventTypeCondition}
                onChange={(condition) => setFormData({ ...formData, eventTypeCondition: condition })}
              />
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Event Version Condition (Optional)</CardTitle>
              <CardDescription>Define conditions for matching event versions</CardDescription>
            </CardHeader>
            <CardContent>
              <ConditionBuilder
                value={formData.eventVersionCondition || defaultCondition}
                onChange={(condition) => setFormData({ ...formData, eventVersionCondition: condition })}
              />
            </CardContent>
          </Card>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={loading}>
              {loading ? 'Saving...' : 'Save Rule'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
} 