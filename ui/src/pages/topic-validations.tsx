import { useEffect, useState } from 'react';
import { getAllValidations, createValidation, deleteValidation, TopicValidationConfig, DataSchema } from '../services/topic-validations';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Trash2 } from 'lucide-react';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { useToast } from "@/hooks/use-toast"
import { ValidationDialog } from '@/components/topic-validations/validation-dialog';

export default function TopicValidationsPage() {
  const { toast } = useToast()
  const [validations, setValidations] = useState<Record<string, TopicValidationConfig[]>>({});
  const [isDialogOpen, setIsDialogOpen] = useState(false);

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

  const handleSave = async (validation: Omit<TopicValidationConfig, 'id'>) => {
    try {
      await createValidation(validation);
      toast({
        title: "Success",
        description: "Validation schema created successfully",
      });
      await loadValidations();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create validation schema",
        variant: "destructive",
      });
      throw error;
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
                          <h3 className="font-medium">{schema.schema.name}</h3>
                          <div className="text-sm text-muted-foreground space-y-1">
                            <p>Event Type: {schema.schema.event_type}</p>
                            <p>Event Version: {schema.schema.event_version}</p>
                            {schema.schema.description && (
                              <p>{schema.schema.description}</p>
                            )}
                          </div>
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

      <ValidationDialog
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        onSave={handleSave}
        onSuccess={loadValidations}
      />
    </div>
  );
} 