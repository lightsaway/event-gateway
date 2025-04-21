import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Plus, X, ChevronDown, ChevronRight } from 'lucide-react';
import { Condition } from '@/services/routing-rules';

interface ConditionBuilderProps {
  value: Condition;
  onChange: (condition: Condition) => void;
}

const expressionTypes = [
  { value: 'regexMatch', label: 'Regex Match' },
  { value: 'equals', label: 'Equals' },
  { value: 'startsWith', label: 'Starts With' },
  { value: 'endsWith', label: 'Ends With' },
  { value: 'contains', label: 'Contains' },
];

const operatorTypes = [
  { value: 'and', label: 'AND' },
  { value: 'or', label: 'OR' },
  { value: 'not', label: 'NOT' },
];

export function ConditionBuilder({ value, onChange }: ConditionBuilderProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const handleAddCondition = () => {
    if (!value.and && !value.or) {
      // If it's a leaf node, convert it to an AND group
      onChange({ and: [value, { type: 'equals', value: '' }] });
    } else {
      // Add to existing group
      const group = value.and ? 'and' : 'or';
      const conditions = value[group] || [];
      onChange({
        ...value,
        [group]: [...conditions, { type: 'equals', value: '' }],
      });
    }
  };

  const handleRemoveCondition = (index: number) => {
    const group = value.and ? 'and' : 'or';
    const conditions = value[group] || [];
    conditions.splice(index, 1);
    
    if (conditions.length === 0) {
      onChange({ type: 'equals', value: '' });
    } else if (conditions.length === 1) {
      onChange(conditions[0]);
    } else {
      onChange({ [group]: conditions });
    }
  };

  const handleChangeOperator = (newOperator: string) => {
    if (newOperator === 'and' || newOperator === 'or') {
      const conditions = value.and || value.or || [{ type: 'equals', value: '' }];
      onChange({ [newOperator]: conditions });
    } else if (newOperator === 'not') {
      onChange({ not: value });
    }
  };

  // Handle single expression (leaf node)
  if (value.type && value.value !== undefined) {
    return (
      <div className="flex items-center gap-2 p-2 border rounded-md">
        <Select
          value={value.type}
          onValueChange={(newType) => onChange({ ...value, type: newType })}
        >
          <SelectTrigger className="w-[140px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {expressionTypes.map((type) => (
              <SelectItem key={type.value} value={type.value}>
                {type.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Input
          value={value.value}
          onChange={(e) => onChange({ ...value, value: e.target.value })}
          placeholder="Value"
          className="flex-1"
        />
        <Button
          variant="ghost"
          size="icon"
          onClick={handleAddCondition}
          title="Add another condition"
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
    );
  }

  // Handle group node (AND/OR/NOT)
  const group = value.and ? 'and' : value.or ? 'or' : 'not';
  const conditions = (value[group] || []) as Condition[];

  return (
    <div className="border rounded-md p-2">
      <div className="flex items-center gap-2 mb-2">
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {isExpanded ? (
            <ChevronDown className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </Button>
        <Select
          value={group}
          onValueChange={handleChangeOperator}
        >
          <SelectTrigger className="w-[100px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {operatorTypes.map((type) => (
              <SelectItem key={type.value} value={type.value}>
                {type.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleAddCondition}
          title="Add another condition"
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      {isExpanded && (
        <div className="space-y-2 pl-6">
          {conditions.map((condition, index) => (
            <div key={index} className="relative">
              <ConditionBuilder
                value={condition}
                onChange={(newCondition) => {
                  const updatedConditions = [...conditions];
                  updatedConditions[index] = newCondition;
                  onChange({ ...value, [group]: updatedConditions });
                }}
              />
              <Button
                variant="ghost"
                size="icon"
                className="absolute -left-6 top-1/2 -translate-y-1/2"
                onClick={() => handleRemoveCondition(index)}
                title="Remove condition"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
} 