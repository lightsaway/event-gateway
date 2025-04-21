import { useEffect, useState } from 'react';
import { getAllValidations, createValidation, deleteValidation, TopicValidationConfig, DataSchema } from '../services/topic-validations';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Textarea } from '../components/ui/textarea';
import { Uuid } from '../types/common';
import { Trash2 } from 'lucide-react';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useToast } from "@/hooks/use-toast"

export default function TopicValidationsPage() {
  const { toast } = useToast()
  const [validations, setValidations] = useState<Record<string, DataSchema[]>>({});
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [selectedValidation, setSelectedValidation] = useState<TopicValidationConfig | null>(null);
  const [formData, setFormData] = useState({
    topic: '',
    schema: {
      id: '',
      name: '',
      schema: '',
      description: '',
    },
  });

  useEffect(() => {
    loadValidations();
  }, []);

  const loadValidations = async () => {
    try {
      const data = await getAllValidations();
      setValidations(data);
    } catch (error) {
      console.error('Failed to load validations:', error);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await createValidation(formData);
      toast({
        title: "Success",
        description: "Validation schema created successfully",
      });
      setIsDialogOpen(false);
      loadValidations();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create validation schema",
        variant: "destructive",
      });
    }
  };

  const handleDelete = async (topic: string, schemaId: string) => {
    try {
      await deleteValidation(schemaId);
      toast({
        title: "Success",
        description: "Validation schema deleted successfully",
      });
      loadValidations();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete validation schema",
        variant: "destructive",
      });
    }
  };

  return (
    <div className="container mx-auto py-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Topic Validations</h1>
        <Button onClick={() => setIsDialogOpen(true)}>Add Validation</Button>
      </div>

      <div className="grid gap-6">
        {Object.entries(validations).map(([topic, schemas]) => (
          <Card key={topic}>
            <CardHeader>
              <CardTitle>{topic}</CardTitle>
              <CardDescription>Validation schemas for this topic</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {schemas.map((schema) => (
                  <Collapsible key={schema.id} className="border rounded-lg overflow-hidden">
                    <div className="flex items-center justify-between">
                      <CollapsibleTrigger className="flex-1 text-left">
                        <div className="p-4">
                          <h3 className="font-medium">{schema.name}</h3>
                          {schema.description && (
                            <p className="text-sm text-gray-500">{schema.description}</p>
                          )}
                        </div>
                      </CollapsibleTrigger>
                      <div className="p-4">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(topic, schema.id)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                    <CollapsibleContent>
                      <div className="px-4 pb-4">
                        <pre className="bg-muted p-4 rounded-lg text-sm overflow-x-auto">
                          {JSON.stringify(schema.schema, null, 2)}
                        </pre>
                      </div>
                    </CollapsibleContent>
                  </Collapsible>
                ))}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Topic Validation</DialogTitle>
            <DialogDescription>
              Create a new validation schema for a topic
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <Label htmlFor="topic">Topic</Label>
              <Input
                id="topic"
                value={formData.topic}
                onChange={(e) => setFormData({ ...formData, topic: e.target.value })}
                required
              />
            </div>
            <div>
              <Label htmlFor="schemaName">Schema Name</Label>
              <Input
                id="schemaName"
                value={formData.schema.name}
                onChange={(e) => setFormData({
                  ...formData,
                  schema: { ...formData.schema, name: e.target.value },
                })}
                required
              />
            </div>
            <div>
              <Label htmlFor="schema">Schema</Label>
              <Textarea
                id="schema"
                value={formData.schema.schema}
                onChange={(e) => setFormData({
                  ...formData,
                  schema: { ...formData.schema, schema: e.target.value },
                })}
                required
              />
            </div>
            <div>
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={formData.schema.description}
                onChange={(e) => setFormData({
                  ...formData,
                  schema: { ...formData.schema, description: e.target.value },
                })}
              />
            </div>
            <DialogFooter>
              <Button type="submit">Create</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
} 