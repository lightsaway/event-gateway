import { useEffect, useState } from 'react';
import { TopicRoutingRule, getAllRules, deleteRule, createRule, updateRule } from '@/services/routing-rules';
import { Button } from '@/components/ui/button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Pencil, Trash2 } from 'lucide-react';
import { RuleDialog } from '@/components/routing-rules/rule-dialog';

export default function RoutingRulesPage() {
  const [rules, setRules] = useState<TopicRoutingRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedRule, setSelectedRule] = useState<TopicRoutingRule | undefined>();

  useEffect(() => {
    loadRules();
  }, []);

  async function loadRules() {
    try {
      setLoading(true);
      const data = await getAllRules();
      setRules(data);
      setError(null);
    } catch (err) {
      setError('Failed to load routing rules');
      console.error(err);
    } finally {
      setLoading(false);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this rule?')) {
      return;
    }

    try {
      await deleteRule(id);
      setRules(rules.filter(rule => rule.id !== id));
    } catch (err) {
      setError('Failed to delete rule');
      console.error(err);
    }
  }

  async function handleSave(rule: Omit<TopicRoutingRule, 'id'>) {
    try {
      if (selectedRule) {
        await updateRule(selectedRule.id, { ...rule, id: selectedRule.id });
        setRules(rules.map(r => r.id === selectedRule.id ? { ...rule, id: selectedRule.id } : r));
      } else {
        const newRule = await createRule(rule);
        setRules([...rules, newRule]);
      }
    } catch (err) {
      throw err;
    }
  }

  function handleEdit(rule: TopicRoutingRule) {
    setSelectedRule(rule);
    setDialogOpen(true);
  }

  function handleAdd() {
    setSelectedRule(undefined);
    setDialogOpen(true);
  }

  if (loading) {
    return <div className="p-4">Loading...</div>;
  }

  if (error) {
    return (
      <div className="p-4 text-red-500">
        Error: {error}
        <Button onClick={loadRules} className="ml-2">
          Retry
        </Button>
      </div>
    );
  }

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Routing Rules</h1>
        <Button onClick={handleAdd}>Add Rule</Button>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Order</TableHead>
            <TableHead>Topic</TableHead>
            <TableHead>Event Type</TableHead>
            <TableHead>Event Version</TableHead>
            <TableHead>Description</TableHead>
            <TableHead className="w-[100px]">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rules.map((rule) => (
            <TableRow key={rule.id}>
              <TableCell>{rule.order}</TableCell>
              <TableCell>{rule.topic}</TableCell>
              <TableCell>{rule.eventTypeCondition.value}</TableCell>
              <TableCell>{rule.eventVersionCondition?.value || '-'}</TableCell>
              <TableCell>{rule.description || '-'}</TableCell>
              <TableCell>
                <div className="flex gap-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleEdit(rule)}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleDelete(rule.id)}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          ))}
          {rules.length === 0 && (
            <TableRow>
              <TableCell colSpan={6} className="text-center">
                No routing rules found
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>

      <RuleDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSave={handleSave}
        initialData={selectedRule}
      />
    </div>
  );
} 