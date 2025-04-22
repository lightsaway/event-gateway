import  { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Input } from '@/components/ui/input';
import { Plus, X, ChevronDown, ChevronRight, Ban } from 'lucide-react';
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
];

export function ConditionBuilder({ value, onChange }: ConditionBuilderProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const handleAddCondition = () => {
    if (!value.and && !value.or) {
      // If it's a leaf node, convert it to an AND group
      onChange({ and: [value, { type: 'equals', value: '' }] });
    } else {
      // Add to existing group
      if (value.and) {
        onChange({
          ...value,
          and: [...value.and, { type: 'equals', value: '' }],
        });
      } else if (value.or) {
        onChange({
          ...value,
          or: [...value.or, { type: 'equals', value: '' }],
        });
      }
    }
  };

  const handleRemoveCondition = (index: number) => {
    if (value.and) {
      const conditions = [...value.and];
      conditions.splice(index, 1);
      
      if (conditions.length === 0) {
        onChange({ type: 'equals', value: '' });
      } else if (conditions.length === 1) {
        onChange(conditions[0]);
      } else {
        onChange({ and: conditions });
      }
    } else if (value.or) {
      const conditions = [...value.or];
      conditions.splice(index, 1);
      
      if (conditions.length === 0) {
        onChange({ type: 'equals', value: '' });
      } else if (conditions.length === 1) {
        onChange(conditions[0]);
      } else {
        onChange({ or: conditions });
      }
    }
  };

  const handleChangeOperator = (newOperator: string) => {
    if (newOperator === 'and' || newOperator === 'or') {
      const conditions = value.and || value.or || [{ type: 'equals', value: '' }];
      onChange({ [newOperator]: conditions });
    }
  };

  const handleToggleNegation = () => {
    if (value.not) {
      // If already negated, remove negation
      onChange(value.not);
    } else {
      // Add negation while preserving the original condition structure
      const originalCondition = { ...value };
      onChange({ not: originalCondition });
    }
  };

  // Handle single expression (leaf node)
  if (value.type && value.value !== undefined) {
    return (
      <div className={`flex items-center gap-2 p-2 border rounded-md ${value.not ? 'border-red-300 bg-red-50' : ''}`}>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleToggleNegation}
          title={value.not ? "Remove NOT" : "Add NOT"}
          className={value.not ? "text-red-500" : ""}
        >
          <Ban className="h-4 w-4" />
        </Button>
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

  // Handle group node (AND/OR)
  const isAndGroup = !!value.and;
  const isOrGroup = !!value.or;
  const conditions = isAndGroup ? value.and : isOrGroup ? value.or : [];

  // If this is a NOT condition, render the inner condition
  if (value.not) {
    return (
      <div className="border rounded-md p-2 border-red-300 bg-red-50">
        <div className="flex items-center gap-2 mb-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={handleToggleNegation}
            title="Remove NOT"
            className="text-red-500"
          >
            <Ban className="h-4 w-4" />
          </Button>
          <span className="font-semibold">NOT</span>
        </div>
        <div className="pl-6">
          <ConditionBuilder
            value={value.not}
            onChange={(newCondition) => onChange({ not: newCondition })}
          />
        </div>
      </div>
    );
  }

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
        <Button
          variant="ghost"
          size="icon"
          onClick={handleToggleNegation}
          title={value.not ? "Remove NOT" : "Add NOT"}
          className={value.not ? "text-red-500" : ""}
        >
          <Ban className="h-4 w-4" />
        </Button>
        <Select
          value={isAndGroup ? 'and' : isOrGroup ? 'or' : ''}
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
      {isExpanded && conditions && (
        <div className="space-y-2 pl-6">
          {conditions.map((condition, index) => (
            <div key={index} className="relative">
              <ConditionBuilder
                value={condition}
                onChange={(newCondition) => {
                  if (isAndGroup) {
                    const updatedConditions = [...value.and!];
                    updatedConditions[index] = newCondition;
                    onChange({ ...value, and: updatedConditions });
                  } else if (isOrGroup) {
                    const updatedConditions = [...value.or!];
                    updatedConditions[index] = newCondition;
                    onChange({ ...value, or: updatedConditions });
                  }
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